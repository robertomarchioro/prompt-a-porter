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

export async function syncLogin(
  serverUrl: string,
  email: string,
  password: string,
): Promise<{ token: string; user: { id: string; displayName: string; role: string } }> {
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

function connettiWs() {
  chiudiWs();
  if (!_config?.serverUrl || !_config?.token) return;

  const wsUrl = _config.serverUrl
    .replace(/\/+$/, "")
    .replace(/^http/, "ws");

  try {
    _ws = new WebSocket(`${wsUrl}/ws?token=${_config.token}`);

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
