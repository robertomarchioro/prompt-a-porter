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

/** Segna il tour come visto (su completamento/chiusura o su "Non ora"). */
export function segnaTourBenvenutoVisto(): void {
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

// Driver attivo: evita due overlay sovrapposti se un tour viene avviato mentre
// un altro è in corso (benvenuto e micro-tour condividono questa variabile).
let driverAttivo: ReturnType<typeof driver> | null = null;

// Tiene solo i passi il cui elemento è effettivamente in pagina (robustezza se
// un pannello non è montato, es. editor senza prompt selezionato o una lista
// vuota al primo accesso).
function passiPresenti(passi: readonly DriveStep[]): DriveStep[] {
  return passi.filter(
    (p) =>
      typeof p.element === "string" &&
      document.querySelector(p.element) !== null,
  );
}

// Quando pre-empiamo un tour in corso per avviarne un altro, il `destroy()` del
// vecchio driver fa scattare il suo `onDestroyed`. Questo flag evita che quel
// callback segni "visto" un tour che l'utente non ha davvero completato.
let preempting = false;

// Costruisce ed esegue il driver per `passi` già filtrati. `alChiudere` segna
// il flag "visto" appropriato (benvenuto o micro-tour). Se non resta alcun
// passo, segna visto senza mostrare nulla (niente overlay vuoto).
function avviaDriver(
  passi: DriveStep[],
  alChiudere: () => void,
  doneBtnText: string,
): void {
  // Chiude un eventuale tour in corso senza farne scattare la marcatura.
  if (driverAttivo) {
    preempting = true;
    driverAttivo.destroy();
    preempting = false;
  }
  driverAttivo = null;
  if (passi.length === 0) {
    // Nessun target montato: segna visto senza overlay. La marcatura è
    // permanente (fino a bump della versione del tour); per i tour attuali le
    // ancore sono sempre presenti quando l'area è montata, quindi non accade.
    alChiudere();
    return;
  }
  driverAttivo = driver({
    showProgress: passi.length > 1,
    progressText: "{{current}} di {{total}}",
    nextBtnText: "Avanti",
    prevBtnText: "Indietro",
    doneBtnText,
    popoverClass: "pap-tour",
    steps: passi,
    onDestroyed: () => {
      if (!preempting) alChiudere();
      driverAttivo = null;
    },
  });
  driverAttivo.drive();
}

/**
 * Esegue il tour di benvenuto. Va chiamato quando gli elementi target sono
 * visibili (nessuna modale aperta sopra lo Shell).
 */
export function eseguiTourBenvenuto(): void {
  if (typeof document === "undefined") return;
  avviaDriver(passiPresenti(PASSI), segnaTourBenvenutoVisto, "Fine");
}

// ─── Micro-tour per-feature (Fase 1 PR-3) ───
// Tour brevi (2 passi) offerti alla PRIMA apertura di un'area avanzata. Stessa
// infrastruttura del benvenuto, con un flag localStorage versionato per id.
export type IdMicroTour = "import";

const VERSIONE_MICRO = 1;

const PASSI_MICRO: Record<IdMicroTour, readonly DriveStep[]> = {
  import: [
    {
      element: '[data-tour="iv-import"]',
      popover: {
        title: "Import componibili",
        description:
          'Gli import {{import "percorso"}} che scrivi nell\'editor compaiono qui: ogni prompt richiamato diventa un mattoncino riusabile. Cliccane uno per aprirlo.',
        side: "bottom",
        align: "start",
      },
    },
    {
      element: '[data-tour="iv-varianti"]',
      popover: {
        title: "Varianti A/B/C",
        description:
          'Crea varianti dello stesso prompt per confrontare formulazioni diverse; "Confronta tutte" le affianca per un A/B diretto.',
        side: "top",
        align: "start",
      },
    },
  ],
};

function chiaveMicro(id: IdMicroTour): string {
  return `pap.tour.micro.${id}.visto`;
}

/** True se il micro-tour `id` (versione corrente) è già stato visto. */
export function microTourVisto(id: IdMicroTour): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(chiaveMicro(id)) === String(VERSIONE_MICRO);
}

function segnaMicroVisto(id: IdMicroTour): void {
  try {
    localStorage.setItem(chiaveMicro(id), String(VERSIONE_MICRO));
  } catch {
    // localStorage pieno o disabilitato: il micro-tour si ri-offrirà.
  }
}

// Esegue il micro-tour `id` (filtra i passi non montati). Privata: l'unico
// ingresso pubblico è `offriMicroTour`, che rispetta il flag "già visto".
function eseguiMicroTour(id: IdMicroTour): void {
  if (typeof document === "undefined") return;
  avviaDriver(
    passiPresenti(PASSI_MICRO[id]),
    () => segnaMicroVisto(id),
    "Ho capito",
  );
}

/**
 * Offre il micro-tour `id` se non ancora visto. Da chiamare nell'`onMount`
 * dell'area: esegue sul frame successivo (doppio rAF) per dare tempo al DOM del
 * pannello di montarsi. Restituisce un cleanup che annulla l'avvio se il
 * componente viene smontato subito (es. cambio tab veloce).
 */
export function offriMicroTour(id: IdMicroTour): () => void {
  if (typeof document === "undefined" || microTourVisto(id)) {
    return () => {};
  }
  let rafOuter = 0;
  let rafInner = 0;
  rafOuter = requestAnimationFrame(() => {
    rafInner = requestAnimationFrame(() => eseguiMicroTour(id));
  });
  // Il cleanup è sincrono (onDestroy): la closure legge i valori correnti di
  // rafOuter/rafInner. Se l'outer ha già girato, rafInner è valorizzato e viene
  // annullato; altrimenti vale 0 e cancelAnimationFrame(0) è un no-op innocuo.
  return () => {
    cancelAnimationFrame(rafOuter);
    cancelAnimationFrame(rafInner);
  };
}
