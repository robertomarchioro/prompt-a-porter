// Helper diff per CronologiaPrompt — Fase 4 Step 3.
//
// Wrap minimale della libreria `diff` (jsdiff, BSD-3-Clause) che
// abbiamo scelto al posto di diff-match-patch perché:
// - già in uso negli ecosistemi Svelte/Vite tipici
// - API più ergonomica (diffWords, diffLines, structuredPatch)
// - tree-shakeable, niente WASM, niente init globale
//
// Le funzioni esposte qui producono strutture di token annotati
// (`Segment`) consumate dal componente `VersionDiff.svelte`. Il
// segnaposto `{{nome}}` resta intatto perché jsdiff agisce su
// boundaries `\b` di parola e i nostri segnaposti finiscono
// naturalmente come token unitari.

import { diffLines, diffWordsWithSpace } from "diff";

/// Tipo singolo token nel risultato.
export type SegmentTipo = "uguale" | "aggiunto" | "rimosso";

export interface Segment {
  tipo: SegmentTipo;
  testo: string;
}

/// Riga annotata per la modalità side-by-side. Ognuno dei due lati
/// può avere `null` se la riga è esclusiva all'altro lato (tipico
/// nelle aggiunte/rimozioni).
export interface RigaSideBySide {
  /// Numero di riga 1-based nel testo `a` (None per righe solo `b`).
  numeroA: number | null;
  /// Numero di riga 1-based nel testo `b` (None per righe solo `a`).
  numeroB: number | null;
  testoA: string | null;
  testoB: string | null;
  /// Stato della riga rispetto al confronto a→b.
  /// - `uguale`: la riga è presente identica in entrambi
  /// - `cambiata`: presente in entrambi ma diversa
  /// - `aggiunta`: solo in `b` (nuova)
  /// - `rimossa`: solo in `a` (cancellata)
  stato: "uguale" | "cambiata" | "aggiunta" | "rimossa";
}

/// Diff a livello di parola (preservando whitespace) per la modalità
/// `unified`/`inline`. Ritorna una sequenza di segmenti consecutivi.
export function diffParole(a: string, b: string): Segment[] {
  const raw = diffWordsWithSpace(a, b);
  return raw.map((part) => ({
    tipo: part.added ? "aggiunto" : part.removed ? "rimosso" : "uguale",
    testo: part.value,
  }));
}

/// Diff per riga (modalità side-by-side). Allinea le righe comuni e
/// restituisce un layout simmetrico A | B. Algoritmo semplice basato
/// su `diffLines`: ogni hunk diff è espanso in righe e accoppiato
/// agli hunk opposti per produrre il pairing.
export function diffSideBySide(a: string, b: string): RigaSideBySide[] {
  const hunks = diffLines(a, b);
  const out: RigaSideBySide[] = [];

  // Buffer per pairing: quando arriva una rimozione, mettiamo le righe
  // in `pendentiA`; alla successiva aggiunta le accoppiamo come
  // "cambiate". Eccedenze diventano `rimossa`/`aggiunta`.
  let pendentiA: { num: number; testo: string }[] = [];
  let numeroA = 1;
  let numeroB = 1;

  function flushPendentiA() {
    for (const p of pendentiA) {
      out.push({
        numeroA: p.num,
        numeroB: null,
        testoA: p.testo,
        testoB: null,
        stato: "rimossa",
      });
    }
    pendentiA = [];
  }

  for (const hunk of hunks) {
    const righe = hunk.value.split("\n");
    // L'ultima riga è vuota se il hunk finisce con \n: scartiamola
    // per evitare un "ghost" finale.
    if (righe.length > 0 && righe[righe.length - 1] === "") {
      righe.pop();
    }

    if (hunk.added) {
      for (const r of righe) {
        const accoppia = pendentiA.shift();
        if (accoppia) {
          out.push({
            numeroA: accoppia.num,
            numeroB: numeroB,
            testoA: accoppia.testo,
            testoB: r,
            stato: "cambiata",
          });
        } else {
          out.push({
            numeroA: null,
            numeroB: numeroB,
            testoA: null,
            testoB: r,
            stato: "aggiunta",
          });
        }
        numeroB++;
      }
    } else if (hunk.removed) {
      for (const r of righe) {
        pendentiA.push({ num: numeroA, testo: r });
        numeroA++;
      }
    } else {
      // hunk uguale: prima flush eventuali rimozioni residue, poi
      // emetti righe identiche.
      flushPendentiA();
      for (const r of righe) {
        out.push({
          numeroA: numeroA,
          numeroB: numeroB,
          testoA: r,
          testoB: r,
          stato: "uguale",
        });
        numeroA++;
        numeroB++;
      }
    }
  }
  flushPendentiA();

  return out;
}

/// Esporta un diff come Markdown unificato in stile patch (`+`/`-`).
/// Pensato per copy-paste in code review fuori dall'app.
export function diffMarkdown(
  a: string,
  b: string,
  intestazione?: { etichettaA: string; etichettaB: string },
): string {
  const linee: string[] = [];
  if (intestazione) {
    linee.push(`--- ${intestazione.etichettaA}`);
    linee.push(`+++ ${intestazione.etichettaB}`);
    linee.push("");
  }
  linee.push("```diff");

  const hunks = diffLines(a, b);
  for (const hunk of hunks) {
    const righe = hunk.value.split("\n");
    if (righe.length > 0 && righe[righe.length - 1] === "") righe.pop();
    const prefix = hunk.added ? "+" : hunk.removed ? "-" : " ";
    for (const r of righe) {
      linee.push(`${prefix}${r}`);
    }
  }

  linee.push("```");
  return linee.join("\n");
}

/// Conteggio righe aggiunte/rimosse per badge UI. Riusa
/// `diffSideBySide` per coerenza coi numeri visualizzati: una riga
/// `cambiata` conta sia come aggiunta che come rimossa.
export function statiDiff(a: string, b: string): { aggiunte: number; rimosse: number } {
  const righe = diffSideBySide(a, b);
  let aggiunte = 0;
  let rimosse = 0;
  for (const r of righe) {
    if (r.stato === "aggiunta") aggiunte++;
    else if (r.stato === "rimossa") rimosse++;
    else if (r.stato === "cambiata") {
      aggiunte++;
      rimosse++;
    }
  }
  return { aggiunte, rimosse };
}
