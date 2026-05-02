export interface Segnaposto {
  nome: string;
  indice: number;
}

const RE_SEGNAPOSTO = /\{\{\s*(\w+)\s*\}\}/g;

export function estraiSegnaposti(body: string): Segnaposto[] {
  const risultati: Segnaposto[] = [];
  const visti = new Set<string>();
  let match;
  RE_SEGNAPOSTO.lastIndex = 0;
  while ((match = RE_SEGNAPOSTO.exec(body)) !== null) {
    if (!visti.has(match[1])) {
      risultati.push({ nome: match[1], indice: match.index });
      visti.add(match[1]);
    }
  }
  return risultati;
}

export function compila(
  body: string,
  valori: Record<string, string>,
): string {
  return body.replace(
    RE_SEGNAPOSTO,
    (_, nome) => valori[nome]?.trim() || `{{${nome}}}`,
  );
}

export function contaCompilati(
  segnaposti: Segnaposto[],
  valori: Record<string, string>,
): number {
  return segnaposti.filter((s) => valori[s.nome]?.trim()).length;
}
