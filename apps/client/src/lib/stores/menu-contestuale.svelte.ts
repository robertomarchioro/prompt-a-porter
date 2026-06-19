/**
 * Store globale del menu contestuale (tasto destro context-aware).
 *
 * Una sola istanza di menu aperta alla volta: aprirne uno sovrascrive lo stato
 * precedente. Effimero, nessuna persistenza. Il primitivo `MenuContestuale`
 * (montato una volta in Shell) legge questo stato; ogni superficie chiama
 * `apriMenu(e.clientX, e.clientY, vociX(ctx))` nel proprio handler
 * `oncontextmenu`. Vedi blueprint `docs/roadmap/menu-contestuale.md`.
 */
export type VoceMenu =
  | {
      id: string;
      label: string;
      /** Icona lucide-svelte opzionale (class component legacy). */
      icona?: typeof import("lucide-svelte").Icon;
      /** Scorciatoia in formato `mod+d`, resa con `fmtShortcut()`. */
      scorciatoia?: string;
      /** Stile distruttivo (rosso) per le azioni pericolose. */
      pericolo?: boolean;
      /** Voce grigia non attivabile (es. backend mancante). */
      disabilitato?: boolean;
      azione?: () => void | Promise<void>;
    }
  | { separatore: true };

export function isSeparatore(v: VoceMenu): v is { separatore: true } {
  return "separatore" in v;
}

interface StatoMenu {
  aperto: boolean;
  x: number;
  y: number;
  voci: VoceMenu[];
}

class StatoMenuContestuale {
  stato = $state<StatoMenu>({ aperto: false, x: 0, y: 0, voci: [] });
}

export const menuContestuale = new StatoMenuContestuale();

/** Apre il menu alle coordinate date con le voci fornite. */
export function apriMenu(x: number, y: number, voci: VoceMenu[]): void {
  menuContestuale.stato = { aperto: true, x, y, voci };
}

/** Chiude il menu (Esc, click fuori, selezione voce). */
export function chiudiMenu(): void {
  if (menuContestuale.stato.aperto) {
    menuContestuale.stato = { ...menuContestuale.stato, aperto: false };
  }
}
