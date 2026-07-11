import { invoke } from "@tauri-apps/api/core";

export type StatoSync = "idle" | "syncing" | "error" | "offline" | "non_configurato";

export interface SyncConfig {
  serverUrl: string;
  email: string;
  token: string;
  intervalloSec: number;
  abilitato: boolean;
}

export interface SyncDelta {
  prompts: SyncPrompt[];
  tags: SyncTag[];
  promptTags: SyncPromptTag[];
  timestamp: string;
}

interface SyncPrompt {
  id: string;
  workspaceId: string;
  authorUserId: string;
  title: string;
  description?: string;
  body: string;
  visibility: string;
  targetModel?: string;
  isFavorite: number;
  useCount: number;
  lastUsedAt?: string;
  version: number;
  createdAt: string;
  updatedAt: string;
  updatedByUserId?: string;
  deletedAt?: string;
}

interface SyncTag {
  id: string;
  workspaceId: string;
  name: string;
  color?: string;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string;
}

interface SyncPromptTag {
  promptId: string;
  tagId: string;
}

export interface SyncState {
  stato: StatoSync;
  ultimoSync: string | null;
  errore: string | null;
  conflitti: number;
}

let _stato: SyncState = {
  stato: "non_configurato",
  ultimoSync: null,
  errore: null,
  conflitti: 0,
};

let _timer: ReturnType<typeof setInterval> | null = null;
let _ws: WebSocket | null = null;
let _config: SyncConfig | null = null;
let _onChange: (() => void) | null = null;

export function syncGetState(): SyncState {
  return { ..._stato };
}

export function syncOnChange(cb: () => void) {
  _onChange = cb;
}

function notifica() {
  _onChange?.();
}

function aggiornaStato(partial: Partial<SyncState>) {
  Object.assign(_stato, partial);
  notifica();
}

/**
 * Fix #455 (security review MEDIUM): il server di sync riceve password
 * di login e token — mai in chiaro sulla rete. Accettiamo solo `https://`.
 * Non esiste un concetto di "dev mode" nel client (a differenza, per
 * esempio, dei provider AI locali come Ollama in `provider_ai.rs`, che
 * girano legittimamente in `http://localhost`): il server di sync è
 * sempre remoto, quindi nessuna eccezione per localhost/http qui.
 */
export function validaServerUrl(url: string): void {
  const messaggio =
    "L'URL del server di sync deve iniziare con https:// (connessione cifrata obbligatoria).";
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    throw new Error(messaggio);
  }
  if (parsed.protocol !== "https:") {
    throw new Error(messaggio);
  }
}

function messaggioErrore(e: unknown): string {
  return e instanceof Error ? e.message : "Errore sconosciuto";
}

export async function syncLogin(
  serverUrl: string,
  email: string,
  password: string,
): Promise<{ token: string; user: { id: string; displayName: string; role: string } }> {
  validaServerUrl(serverUrl);
  const url = serverUrl.replace(/\/+$/, "");
  const res = await fetch(`${url}/auth/login`, {
    method: "POST",
    // Fix #455 (review LOW-2): un server compromesso non deve poter
    // 30x-redirigere la richiesta di login (quindi la password) verso un
    // altro host, tipicamente in http:// non cifrato.
    redirect: "error",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: "Errore di connessione" }));
    throw new Error(err.error || `HTTP ${res.status}`);
  }

  const data = await res.json();
  return { token: data.token, user: data.user };
}

export async function syncConfigura(config: SyncConfig) {
  // Fix #455: rifiuta config con serverUrl non-https PRIMA di salvarla o
  // usarla, con un messaggio utente chiaro invece di un fetch/WS falliti
  // silenziosamente più avanti.
  validaServerUrl(config.serverUrl);

  _config = config;

  // Fix #455 (review HIGH-2): il token NON passa più dal round-trip
  // generico di `preferenze_carica`/`preferenze_salva` (altrimenti OGNI
  // salvataggio di preferenze non correlate — tema, editor, debug-log, ...
  // — richiederebbe transitivamente il vault aperto). Ha un comando Tauri
  // dedicato, `sync_token_salva`.
  await invoke("preferenze_salva", {
    preferenze: await caricaPreferenzeAggiornate(config),
  });
  await invoke("sync_token_salva", { token: config.token });

  if (config.abilitato && config.token) {
    aggiornaStato({ stato: "idle", errore: null });
    avviaPolling(config.intervalloSec);
    connettiWs();
  } else {
    fermaSync();
    aggiornaStato({ stato: "non_configurato" });
  }
}

export async function syncAvvia(config: SyncConfig) {
  _config = config;
  if (!config.abilitato || !config.token || !config.serverUrl) {
    aggiornaStato({ stato: "non_configurato" });
    return;
  }
  try {
    validaServerUrl(config.serverUrl);
  } catch (e) {
    aggiornaStato({ stato: "error", errore: messaggioErrore(e) });
    return;
  }
  aggiornaStato({ stato: "idle", errore: null });
  avviaPolling(config.intervalloSec);
  connettiWs();
}

export function syncFerma() {
  fermaSync();
  aggiornaStato({ stato: "non_configurato" });
}

export async function syncOra() {
  if (!_config?.token || !_config?.serverUrl) return;
  await eseguiSync();
}

async function eseguiSync() {
  if (!_config) return;
  if (_stato.stato === "syncing") return;

  // Fix #455: ultima linea di difesa prima del fetch — anche se tutti gli
  // ingressi validano già l'URL, `_config` può restare popolato da una
  // chiamata precedente e questa funzione gira anche dal timer di polling.
  try {
    validaServerUrl(_config.serverUrl);
  } catch (e) {
    aggiornaStato({ stato: "error", errore: messaggioErrore(e) });
    return;
  }

  aggiornaStato({ stato: "syncing" });

  const url = _config.serverUrl.replace(/\/+$/, "");
  const headers = {
    Authorization: `Bearer ${_config.token}`,
    "Content-Type": "application/json",
  };

  try {
    const since = _stato.ultimoSync || "1970-01-01 00:00:00";
    const pullRes = await fetch(`${url}/sync/pull?since=${encodeURIComponent(since)}`, { headers });

    if (pullRes.status === 401) {
      aggiornaStato({ stato: "error", errore: "Token scaduto — effettua nuovamente il login" });
      return;
    }

    if (!pullRes.ok) {
      throw new Error(`Pull fallito: HTTP ${pullRes.status}`);
    }

    const delta: SyncDelta = await pullRes.json();

    if (delta.prompts.length > 0 || delta.tags.length > 0) {
      await invoke("sync_applica_delta", { delta });
    }

    aggiornaStato({
      stato: "idle",
      ultimoSync: delta.timestamp,
      errore: null,
    });
  } catch (e) {
    const msg = e instanceof Error ? e.message : "Errore sconosciuto";
    aggiornaStato({ stato: "error", errore: msg });
  }
}

function avviaPolling(intervalloSec: number) {
  fermaPolling();
  _timer = setInterval(() => eseguiSync(), intervalloSec * 1000);
}

function fermaPolling() {
  if (_timer) {
    clearInterval(_timer);
    _timer = null;
  }
}

/**
 * Fix #455 (review HIGH-1): prefisso del sub-protocollo WebSocket usato
 * per veicolare il token. La API `WebSocket` del browser non permette
 * header custom: il secondo parametro del costruttore
 * (`Sec-WebSocket-Protocol`) è l'UNICO canale disponibile, quindi è anche
 * l'UNICO modo in cui il token viaggia (niente più `?token=` in query
 * string — visibile in log di proxy/server e nella history del browser).
 *
 * Formato ESATTO inviato (deve combaciare byte-per-byte con quanto legge
 * il server, apps/server/internal/ws/hub.go, PR #480 —
 * fix/sync-server-hardening): un solo valore nell'header
 * `Sec-WebSocket-Protocol`, pari al prefisso letterale seguito dal JWT,
 * senza separatori né altri protocolli offerti:
 *
 *   Sec-WebSocket-Protocol: pap.sync.token.<JWT>
 *
 * Se questo prefisso cambia, deve cambiare in lockstep sui due lati.
 */
const WS_PROTOCOLLO_TOKEN_PREFIX = "pap.sync.token.";

/**
 * Fix #455 (review MEDIUM-2): deriva l'URL WebSocket (`wss://.../ws`) da
 * un `serverUrl` `https://` usando l'API `URL` (case-insensitive sullo
 * schema) invece di un replace testuale su `/^https/` (che non matcha
 * `HTTPS://`, producendo un `wss` malformato).
 */
function urlWebSocket(serverUrl: string): string {
  const u = new URL(serverUrl);
  u.protocol = "wss:";
  return `${u.toString().replace(/\/+$/, "")}/ws`;
}

function connettiWs() {
  chiudiWs();
  if (!_config?.serverUrl || !_config?.token) return;

  // Fix #455: niente WS verso un serverUrl non validato (coerente con
  // fetch()/syncLogin — vedi validaServerUrl).
  let wsUrl: string;
  try {
    validaServerUrl(_config.serverUrl);
    wsUrl = urlWebSocket(_config.serverUrl);
  } catch (e) {
    console.error("[sync] serverUrl non valido per WebSocket", messaggioErrore(e));
    return;
  }

  try {
    // Fix #455 (review HIGH-1): il token viaggia SOLO nel sub-protocollo
    // (vedi doc-comment di WS_PROTOCOLLO_TOKEN_PREFIX) — nessun `?token=`
    // in query string, nemmeno come fallback temporaneo: il server sync
    // non è ancora deployato, quindi non c'è un contratto legacy da
    // preservare (PR server #480 aggiunge la lettura da
    // Sec-WebSocket-Protocol prima/insieme a questa).
    _ws = new WebSocket(wsUrl, [`${WS_PROTOCOLLO_TOKEN_PREFIX}${_config.token}`]);

    _ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === "sync_update") {
          eseguiSync();
        }
      } catch {
        /* messaggio non JSON */
      }
    };

    _ws.onclose = () => {
      pianificaRiconnessioneWs();
    };

    _ws.onerror = () => {
      /* onclose gestirà il reconnect */
    };
  } catch (e) {
    // Fix #455 (review MEDIUM-1): il costruttore `WebSocket` può lanciare
    // (es. `SyntaxError` se il sub-protocollo contenesse caratteri non
    // validi per un token HTTP). Prima veniva inghiottito in silenzio,
    // disabilitando il real-time per sempre senza log né retry — un
    // futuro cambio di formato del token avrebbe rotto tutto senza
    // nessun segnale. Ora logghiamo, esponiamo l'errore in `SyncState` e
    // ritentiamo con lo stesso backoff usato per `onclose`.
    console.error("[sync] apertura WebSocket fallita", messaggioErrore(e));
    aggiornaStato({
      stato: "error",
      errore: `Connessione realtime non disponibile: ${messaggioErrore(e)}`,
    });
    pianificaRiconnessioneWs();
  }
}

function pianificaRiconnessioneWs() {
  setTimeout(() => {
    if (_config?.abilitato) connettiWs();
  }, 5000);
}

function chiudiWs() {
  if (_ws) {
    _ws.onclose = null;
    _ws.close();
    _ws = null;
  }
}

function fermaSync() {
  fermaPolling();
  chiudiWs();
}

async function caricaPreferenzeAggiornate(config: SyncConfig) {
  // Fix #455 (review HIGH-2): `sync_token` non fa più parte dell'oggetto
  // `Preferenze` — vive nel vault via i comandi dedicati `sync_token_carica`
  // / `sync_token_salva` (vedi `syncConfigura`/`syncLogout`).
  const prefs = await invoke<Record<string, unknown>>("preferenze_carica");
  return {
    ...prefs,
    sync_server_url: config.serverUrl,
    sync_email: config.email,
    sync_intervallo_sec: config.intervalloSec,
    sync_abilitato: config.abilitato,
  };
}

export async function syncLogout() {
  fermaSync();
  _config = null;
  aggiornaStato({
    stato: "non_configurato",
    ultimoSync: null,
    errore: null,
    conflitti: 0,
  });

  const prefs = await invoke<Record<string, unknown>>("preferenze_carica");
  await invoke("preferenze_salva", {
    preferenze: {
      ...prefs,
      sync_server_url: "",
      sync_email: "",
      sync_abilitato: false,
    },
  });
  // Fix #455 (review HIGH-2): pulizia esplicita del token dal vault via il
  // comando dedicato — non più tramite il round-trip generico.
  await invoke("sync_token_salva", { token: "" });
}
