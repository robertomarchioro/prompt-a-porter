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

  await invoke("preferenze_salva", {
    preferenze: await caricaPreferenzeAggiornate(config),
  });

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
 * Fix #455: prefisso del sub-protocollo WebSocket usato per veicolare il
 * token — la API `WebSocket` del browser non permette header custom, il
 * secondo parametro del costruttore (`Sec-WebSocket-Protocol`) è l'unico
 * canale disponibile per farlo senza metterlo in query string.
 */
const WS_PROTOCOLLO_TOKEN_PREFIX = "pap.sync.token.";

function connettiWs() {
  chiudiWs();
  if (!_config?.serverUrl || !_config?.token) return;

  // Fix #455: niente WS verso un serverUrl non validato (coerente con
  // fetch()/syncLogin — vedi validaServerUrl).
  try {
    validaServerUrl(_config.serverUrl);
  } catch {
    return;
  }

  const wsUrl = _config.serverUrl.replace(/\/+$/, "").replace(/^https/, "wss");

  try {
    // Fix #455 (security review MEDIUM): il token non deve mai finire in
    // query string (log del server/proxy, history del browser). Lo
    // inviamo come sub-protocollo via `Sec-WebSocket-Protocol` (secondo
    // argomento di `WebSocket`).
    //
    // TODO(#453): il fallback `?token=` in query string resta SOLO finché
    // il server (apps/server/internal/ws/hub.go, branch
    // fix/sync-server-hardening) non legge il token da
    // Sec-WebSocket-Protocol — serve a non rompere la connessione durante
    // la finestra in cui questa PR e quella lato server non sono ancora
    // entrambe mergiate. Da rimuovere quando #453 è chiuso.
    _ws = new WebSocket(`${wsUrl}/ws?token=${encodeURIComponent(_config.token)}`, [
      `${WS_PROTOCOLLO_TOKEN_PREFIX}${_config.token}`,
    ]);

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
      setTimeout(() => {
        if (_config?.abilitato) connettiWs();
      }, 5000);
    };

    _ws.onerror = () => {
      /* onclose gestirà il reconnect */
    };
  } catch {
    /* WS non disponibile */
  }
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
  const prefs = await invoke<Record<string, unknown>>("preferenze_carica");
  return {
    ...prefs,
    sync_server_url: config.serverUrl,
    sync_email: config.email,
    sync_token: config.token,
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
      sync_token: "",
      sync_abilitato: false,
    },
  });
}
