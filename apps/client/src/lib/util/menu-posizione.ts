// Posizionamento del menu contestuale: funzione pura (testabile) che clampa
// la posizione al viewport e flippa se il menu sborda. Niente DOM, niente
// libreria di floating — vedi blueprint menu-contestuale §3.6.

export interface PuntoMenu {
  left: number;
  top: number;
}

/**
 * Calcola la posizione (left/top in px) del menu a partire dal punto di
 * apertura (cursore o rect dell'elemento), date le dimensioni del menu e del
 * viewport.
 *
 * - Apre verso il basso-destra dal punto.
 * - Se sborda a destra → flippa a sinistra del punto.
 * - Se sborda in basso → flippa sopra il punto.
 * - Clamp finale entro `[margine, viewport - dim - margine]` (per menu più
 *   grandi del viewport resta a `margine`, con scroll interno via CSS).
 */
export function posizionaMenu(
  x: number,
  y: number,
  larghezza: number,
  altezza: number,
  viewportW: number,
  viewportH: number,
  margine = 8,
): PuntoMenu {
  let left = x;
  let top = y;

  if (left + larghezza + margine > viewportW) {
    left = x - larghezza;
  }
  if (top + altezza + margine > viewportH) {
    top = y - altezza;
  }

  left = Math.max(margine, Math.min(left, viewportW - larghezza - margine));
  top = Math.max(margine, Math.min(top, viewportH - altezza - margine));

  return { left, top };
}
