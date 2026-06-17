// Guida interattiva — Fase 1: tour spotlight di benvenuto.
// Incapsula driver.js dietro un'API piccola e stabile. I passi ancorano gli
// elementi via attributi `data-tour="..."` (contratto stabile, indipendente
// da CSS/struttura), così i refactor UI non rompono il tour.
import { driver, type DriveStep } from "driver.js";
import "driver.js/dist/driver.css";
import "./tour.css";

// Versione del tour: aumentandola si ri-offre il tour a chi l'ha già visto
// (utile dopo cambi UI importanti).
const VERSIONE = 1;
const KEY_VISTO = "pap.tour.benvenuto.visto";

/** True se l'utente ha già visto/completato il tour della versione corrente. */
export function tourBenvenutoVisto(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(KEY_VISTO) === String(VERSIONE);
}

function segnaVisto(): void {
  try {
    localStorage.setItem(KEY_VISTO, String(VERSIONE));
  } catch {
    // localStorage pieno o disabilitato: amen, il tour si ri-offrirà.
  }
}

/** Azzera il flag (per il pulsante "rivedi il tour"). */
export function reimpostaTourBenvenuto(): void {
  try {
    localStorage.removeItem(KEY_VISTO);
  } catch {
    // ignore
  }
}

// Richiesta reattiva: l'hub Guida vive dentro la modale Impostazioni, che
// coprirebbe gli elementi da evidenziare. Il pulsante "Avvia il tour" si limita
// a *richiedere* il tour; lo Shell chiude la modale e poi esegue (vedi
// `tourRichiesta` + `consumaRichiesta`).
let _richiesta = $state(false);
export const tourRichiesta = {
  get attiva(): boolean {
    return _richiesta;
  },
};
export function richiediTourBenvenuto(): void {
  _richiesta = true;
}
export function consumaRichiesta(): void {
  _richiesta = false;
}

const PASSI: readonly DriveStep[] = [
  {
    element: '[data-tour="sidebar"]',
    popover: {
      title: "Le tue viste",
      description:
        "Qui trovi tutti i prompt, organizzati per cartelle e tag, più il cestino e le viste speciali.",
      side: "right",
      align: "start",
    },
  },
  {
    element: '[data-tour="nuovo-prompt"]',
    popover: {
      title: "Crea un prompt",
      description: "Da qui parti per scrivere un nuovo prompt nella libreria.",
      side: "bottom",
      align: "start",
    },
  },
  {
    element: '[data-tour="ricerca"]',
    popover: {
      title: "Cerca",
      description:
        "Cerca per parola chiave o per significato: la ricerca è anche semantica.",
      side: "bottom",
      align: "start",
    },
  },
  {
    element: '[data-tour="editor"]',
    popover: {
      title: "L'editor",
      description:
        'Qui scrivi il prompt. Usa i segnaposti come {{nome}} e gli import {{import "..."}} per comporre. I badge ? aprono la guida della singola funzione.',
      side: "left",
      align: "start",
    },
  },
  {
    element: '[data-tour="aiuto"]',
    popover: {
      title: "Serve aiuto?",
      description:
        "Da qui apri sempre la Guida con i link alla documentazione. Buon lavoro!",
      side: "bottom",
      align: "end",
    },
  },
];

// Driver attivo: evita due overlay sovrapposti se il tour viene avviato due
// volte di fila (es. doppio click su "Avvia").
let driverAttivo: ReturnType<typeof driver> | null = null;

/**
 * Esegue il tour di benvenuto. Va chiamato quando gli elementi target sono
 * visibili (nessuna modale aperta sopra lo Shell).
 */
export function eseguiTourBenvenuto(): void {
  if (typeof document === "undefined") return;
  // Annulla un eventuale tour già in corso prima di avviarne un altro.
  driverAttivo?.destroy();

  // Tiene solo i passi il cui elemento è effettivamente in pagina (robustezza
  // se un pannello non è montato, es. editor senza prompt selezionato).
  const passi = PASSI.filter(
    (p) =>
      typeof p.element === "string" &&
      document.querySelector(p.element) !== null,
  );

  driverAttivo = driver({
    showProgress: true,
    progressText: "{{current}} di {{total}}",
    nextBtnText: "Avanti",
    prevBtnText: "Indietro",
    doneBtnText: "Fine",
    popoverClass: "pap-tour",
    steps: passi,
    onDestroyed: () => {
      segnaVisto();
      driverAttivo = null;
    },
  });
  driverAttivo.drive();
}
