// Runner dell'aggiornamento del registro modelli.
//
// Fa SOLO rete e filesystem: filtro, fusione e controlli di sanità stanno in
// `src/lib/modelli-aggiornamento.ts`, dove sono testati.
//
//   pnpm --filter @pap/client modelli:aggiorna          # riscrive il registro
//   pnpm --filter @pap/client modelli:aggiorna --check  # esce 1 se ci sono novità
//
// Esce 0 e non scrive nulla se il registro è già aggiornato; esce 2 su errore
// (rete, schema cambiato, listino sospetto) senza toccare il file.

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import {
  costruisciRegistro,
  descriviDelta,
  type ListinoGrezzo,
} from "../src/lib/modelli-aggiornamento";
import type { RegistroModelli } from "../src/lib/modelli-provider";

const FONTE = "https://models.dev/api.json";
const TIMEOUT_MS = 30_000;

const QUI = dirname(fileURLToPath(import.meta.url));
const PERCORSO_REGISTRO = resolve(QUI, "../src/lib/modelli-registro.json");

function leggiRegistro(): RegistroModelli {
  try {
    return JSON.parse(readFileSync(PERCORSO_REGISTRO, "utf8")) as RegistroModelli;
  } catch {
    // Primo giro: registro non ancora esistente.
    return { aggiornato_a: "", fonte: FONTE, modelli: [] };
  }
}

async function scaricaListino(): Promise<ListinoGrezzo> {
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), TIMEOUT_MS);
  try {
    const r = await fetch(FONTE, { signal: ctrl.signal });
    if (!r.ok) throw new Error(`${FONTE} ha risposto ${r.status}`);
    return (await r.json()) as ListinoGrezzo;
  } finally {
    clearTimeout(t);
  }
}

async function main(): Promise<number> {
  const soloControllo = process.argv.includes("--check");
  const oggi = new Date().toISOString().slice(0, 10);

  const corrente = leggiRegistro();
  const listino = await scaricaListino();

  const { registro, errore } = costruisciRegistro(corrente, listino, oggi);
  if (errore) {
    console.error(`✗ ${errore}`);
    return 2;
  }

  const delta = descriviDelta(corrente.modelli, registro.modelli);
  const cambiato =
    delta.nuovi.length > 0 || delta.obsoleti.length > 0 || delta.tornati.length > 0;

  if (!cambiato) {
    console.log(`✓ Registro già aggiornato — ${registro.modelli.length} modelli.`);
    return 0;
  }

  for (const [titolo, voci] of [
    ["Nuovi", delta.nuovi],
    ["Diventati obsoleti", delta.obsoleti],
    ["Tornati nel listino", delta.tornati],
  ] as const) {
    if (voci.length > 0) console.log(`${titolo} (${voci.length}): ${voci.join(", ")}`);
  }

  if (soloControllo) return 1;

  writeFileSync(PERCORSO_REGISTRO, `${JSON.stringify(registro, null, 2)}\n`, "utf8");
  console.log(`✓ Registro riscritto — ${registro.modelli.length} modelli.`);
  return 0;
}

main().then(
  (codice) => process.exit(codice),
  (e: unknown) => {
    console.error(`✗ ${e instanceof Error ? e.message : String(e)}`);
    process.exit(2);
  },
);
