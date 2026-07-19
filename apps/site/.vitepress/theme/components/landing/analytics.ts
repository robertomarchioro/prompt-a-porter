// Eventi Matomo della landing (istruzioni §3.4). Il tag è iniettato solo
// in produzione (config.ts): qui si accoda su window._paq se esiste, e in
// dev/SSR è un no-op. MAI dati personali negli eventi.
//
// ATTENZIONE: quando matomo.js si carica, sostituisce _paq (array) con un
// TrackerProxy — un oggetto con .push() che applica subito. Il check
// corretto è quindi "ha un push?", NON Array.isArray: con quello gli
// eventi partivano solo finché il tracker NON era ancora caricato (bug
// storico: pageview tracciate, eventi mai).

interface PaqLike {
  push: (...args: unknown[][]) => unknown
}

function paq(): PaqLike | null {
  if (typeof window === 'undefined') return null
  const p = (window as unknown as { _paq?: unknown })._paq
  if (p && typeof (p as PaqLike).push === 'function') return p as PaqLike
  return null
}

export function traccia(categoria: string, azione: string, nome?: string): void {
  const p = paq()
  if (!p) return
  p.push(nome === undefined
    ? ['trackEvent', categoria, azione]
    : ['trackEvent', categoria, azione, nome])
}

/** Pageview manuale (navigazioni SPA) — stesso guard del proxy. */
export function tracciaPagina(url: string, titolo: string): void {
  const p = paq()
  if (!p) return
  p.push(['setCustomUrl', url])
  p.push(['setDocumentTitle', titolo])
  p.push(['trackPageView'])
}
