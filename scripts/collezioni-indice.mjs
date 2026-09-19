#!/usr/bin/env node
// Rigenera `docs/collezioni/indice.json`: per ogni collezione ricalcola il
// numero di prompt e lo sha256 del file, preservando titolo e descrizione
// scritti a mano. Un file senza voce nell'indice viene aggiunto con titolo
// vuoto (da compilare). Il client verifica lo sha256 dopo il download:
// un indice non rigenerato = collezione rifiutata con «file cambiato».
//
// Uso: node scripts/collezioni-indice.mjs [--check]
//   --check  non scrive: esce 1 se l'indice è stale (per la CI).

import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const DIR = join(import.meta.dirname, "..", "docs", "collezioni");
const INDICE = join(DIR, "indice.json");
const SLUG_VALIDO = /^[a-z0-9-]{1,40}$/;

const soloCheck = process.argv.includes("--check");
const indice = JSON.parse(readFileSync(INDICE, "utf8"));
const perSlug = new Map(indice.collezioni.map((c) => [c.slug, c]));

const file = readdirSync(DIR)
  .filter((f) => f.endsWith(".json") && f !== "indice.json")
  .sort();

const collezioni = file.map((nome) => {
  const slug = nome.replace(/\.json$/, "");
  if (!SLUG_VALIDO.test(slug)) {
    throw new Error(`slug non valido: ${nome} (ammessi [a-z0-9-], max 40)`);
  }
  const bytes = readFileSync(join(DIR, nome));
  const vault = JSON.parse(bytes.toString("utf8"));
  const esistente = perSlug.get(slug) ?? { titolo: "", descrizione: "" };
  return {
    slug,
    titolo: esistente.titolo,
    descrizione: esistente.descrizione,
    prompt: vault.prompts.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
});

const nuovo = { schemaVersion: 1, collezioni };
const testo = `${JSON.stringify(nuovo, null, 2)}\n`;

if (soloCheck) {
  if (testo !== readFileSync(INDICE, "utf8")) {
    console.error("indice.json non aggiornato: esegui `node scripts/collezioni-indice.mjs`");
    process.exit(1);
  }
  console.log("indice.json aggiornato");
} else {
  writeFileSync(INDICE, testo);
  for (const c of collezioni) {
    console.log(`${c.slug}: ${c.prompt} prompt, sha256 ${c.sha256.slice(0, 12)}…`);
    if (!c.titolo) console.warn(`  ⚠ titolo vuoto per ${c.slug}: compilalo in indice.json`);
  }
}
