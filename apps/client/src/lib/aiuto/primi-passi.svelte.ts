// Guida interattiva — Fase 3: checklist "Primi passi".
//
// Traccia le 5 prime azioni chiave dell'utente tramite flag in localStorage,
// settati nei PUNTI D'AZIONE reali (creazione prompt, aggiunta tag, salvataggio
// con segnaposto/import, compilazione) — NON derivati dallo stato del DB. Così
// importare il `demo-vault.json` in onboarding non pre-spunta nulla: la
// checklist insegna facendo davvero. Widget dismissibile (vedi PrimiPassi.svelte).

export type IdPasso = "crea" | "tag" | "segnaposto" | "compila" | "import";

export const PASSI: readonly { id: IdPasso; label: string }[] = [
  { id: "crea", label: "Crea un prompt" },
  { id: "tag", label: "Aggiungi un tag" },
  { id: "segnaposto", label: "Usa un segnaposto {{nome}}" },
  { id: "compila", label: "Compila un prompt" },
  { id: "import", label: 'Componi con un {{import "…"}}' },
];

const chiave = (id: IdPasso): string => `pap.primipassi.${id}`;
const KEY_CHIUSA = "pap.primipassi.chiusa";

function leggiFlag(id: IdPasso): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(chiave(id)) === "1";
}

function leggiChiusa(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(KEY_CHIUSA) === "1";
}

let _fatti = $state<Record<IdPasso, boolean>>({
  crea: leggiFlag("crea"),
  tag: leggiFlag("tag"),
  segnaposto: leggiFlag("segnaposto"),
  compila: leggiFlag("compila"),
  import: leggiFlag("import"),
});
let _chiusa = $state(leggiChiusa());

export const primiPassi = {
  get fatti(): Readonly<Record<IdPasso, boolean>> {
    return _fatti;
  },
  get chiusa(): boolean {
    return _chiusa;
  },
  get completati(): number {
    return PASSI.filter((p) => _fatti[p.id]).length;
  },
  get totale(): number {
    return PASSI.length;
  },
  get tutti(): boolean {
    return PASSI.every((p) => _fatti[p.id]);
  },
};

/** Segna un passo come completato (idempotente). Chiamato dai punti d'azione. */
export function segnaPasso(id: IdPasso): void {
  if (_fatti[id]) return;
  _fatti = { ..._fatti, [id]: true };
  try {
    localStorage.setItem(chiave(id), "1");
  } catch {
    // localStorage pieno o disabilitato: la spunta resta solo in memoria.
  }
}

/** Chiude (dismiss) il widget; non si ripropone. */
export function chiudiPrimiPassi(): void {
  _chiusa = true;
  try {
    localStorage.setItem(KEY_CHIUSA, "1");
  } catch {
    // ignore
  }
}

/** Reset completo per il "rivedi i primi passi" dall'hub Guida e aiuto. */
export function reimpostaPrimiPassi(): void {
  // Reset in-memory PRIMA (UI reattiva subito); la pulizia di localStorage è
  // best-effort: se fallisce parzialmente, almeno la sessione corrente è coerente.
  _fatti = {
    crea: false,
    tag: false,
    segnaposto: false,
    compila: false,
    import: false,
  };
  _chiusa = false;
  try {
    for (const p of PASSI) localStorage.removeItem(chiave(p.id));
    localStorage.removeItem(KEY_CHIUSA);
  } catch {
    // ignore
  }
}
