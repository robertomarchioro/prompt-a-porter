// Eventi Matomo della landing (istruzioni §3.4). Il tag è iniettato solo
// in produzione (config.ts): qui si accoda su window._paq se esiste, e in
// dev/SSR è un no-op. MAI dati personali negli eventi.

type Paq = unknown[][]

export function traccia(categoria: string, azione: string, nome?: string): void {
  if (typeof window === 'undefined') return
  const paq = (window as unknown as { _paq?: Paq })._paq
  if (!Array.isArray(paq)) return
  paq.push(nome === undefined
    ? ['trackEvent', categoria, azione]
    : ['trackEvent', categoria, azione, nome])
}
