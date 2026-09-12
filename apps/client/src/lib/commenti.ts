/**
 * Commenti inline nel body del prompt (issue #643): `{{!-- testo --}}`.
 *
 * Il commento è una nota dell'autore che non deve mai raggiungere il
 * modello né gli appunti: viene tolto PRIMA di ogni altra elaborazione del
 * body (segnaposti, import). Resta invece nel vault, nelle versioni,
 * nell'export e nella ricerca, perché è parte del sorgente.
 *
 * Semantica condivisa con MCP server (TS), Rust e Go CLI, verificata dalla
 * fixture `packages/shared-schema/fixtures/commenti-conformita.json`:
 * - unica forma `{{!-- … --}}`, chiude al primo `--}}`, multiriga;
 * - `{{! … }}` NON è un commento; un commento non chiuso resta inalterato;
 * - passo 1: si toglie il token; passo 2: ogni riga che conteneva un
 *   commento e che dopo il passo 1 è vuota (solo spazi/tab/CR) sparisce con
 *   il suo a-capo. Le righe già vuote in origine restano.
 */

const RE_COMMENTO = /\{\{!--[\s\S]*?--\}\}/g;

/** Caratteri che rendono "vuota" una riga dopo la rimozione del token. */
const RE_RIGA_VUOTA = /^[ \t\r]*$/;

export interface IntervalloCommento {
  /** Offset di inizio del token `{{!--` (incluso). */
  from: number;
  /** Offset di fine del token `--}}` (escluso). */
  to: number;
}

/** Posizioni [from, to) di ogni commento nel testo, in ordine. */
export function intervalliCommenti(testo: string): IntervalloCommento[] {
  const re = new RegExp(RE_COMMENTO.source, "g");
  const out: IntervalloCommento[] = [];
  let m: RegExpExecArray | null;
  while ((m = re.exec(testo)) !== null) {
    out.push({ from: m.index, to: m.index + m[0].length });
  }
  return out;
}

function contaNewline(s: string): number {
  let n = 0;
  for (let i = 0; i < s.length; i++) {
    if (s.charCodeAt(i) === 10) n++;
  }
  return n;
}

/** Restituisce il testo senza i commenti `{{!-- … --}}`. */
export function rimuoviCommenti(testo: string): string {
  const intervalli = intervalliCommenti(testo);
  if (intervalli.length === 0) return testo;

  // Passo 1: togli i token, ricordando su quale riga dell'output cadevano.
  const pezzi: string[] = [];
  const righeToccate = new Set<number>();
  let rigaCorrente = 0;
  let cursore = 0;
  for (const { from, to } of intervalli) {
    const pezzo = testo.slice(cursore, from);
    pezzi.push(pezzo);
    rigaCorrente += contaNewline(pezzo);
    righeToccate.add(rigaCorrente);
    cursore = to;
  }
  pezzi.push(testo.slice(cursore));

  // Passo 2: elimina le righe toccate rimaste vuote.
  const righe = pezzi.join("").split("\n");
  return righe
    .filter((riga, i) => !(righeToccate.has(i) && RE_RIGA_VUOTA.test(riga)))
    .join("\n");
}
