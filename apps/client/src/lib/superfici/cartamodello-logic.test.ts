/**
 * Test per cartamodello-logic.ts: scelte iniziali, motivi bloccanti,
 * costruzione della richiesta e riscrittura degli import rinominati.
 */

import { describe, it, expect } from "vitest";
import {
  conScelta,
  conTitoloPrompt,
  costruisciRichiesta,
  formattaEsito,
  motiviBloccanti,
  riscriviImportRinominati,
  scelteIniziali,
  scelteInizialiPrompt,
  type ModuloVerificato,
  type Proposta,
  type PromptVerificato,
} from "./cartamodello-logic";

function modulo(over: Partial<ModuloVerificato> = {}): ModuloVerificato {
  return {
    chiave: "ruolo",
    titolo: "Ruolo senior engineer",
    tipo: "ruolo",
    corpo: "Sei un senior engineer.",
    usato_da: ["p1"],
    problemi: [],
    riuso: null,
    titolo_in_conflitto: null,
    ...over,
  };
}

function prompt(over: Partial<PromptVerificato> = {}): PromptVerificato {
  return {
    id: "p1",
    titolo_originale: "Review",
    titolo: "Code review",
    descrizione: null,
    corpo: '{{import "Ruolo senior engineer"}}\n{{codice}}',
    corpo_originale: "Sei un senior engineer.\n{{codice}}",
    anteprima_espansa: "Sei un senior engineer.\n{{codice}}",
    import_non_risolti: [],
    problemi: [],
    segnaposti: ["codice"],
    ...over,
  };
}

function proposta(over: Partial<Proposta> = {}): Proposta {
  return {
    moduli: [modulo()],
    prompt: [prompt()],
    note: [],
    avvisi: [],
    riuso_semantico: false,
    cartella_moduli: "/Moduli",
    tokens_used: null,
    costo_stimato: null,
    provider: "anthropic",
    model: "claude-sonnet-5",
    troncato: false,
    ...over,
  };
}

describe("scelteIniziali", () => {
  it("propone «crea» senza riuso e «riusa» quando il vault ha un equivalente", () => {
    const p = proposta({
      moduli: [
        modulo(),
        modulo({
          chiave: "formato",
          titolo: "Formato JSON",
          riuso: { prompt_id: "x", titolo: "Rispondi in JSON", similarita: 0.9, fonte: "lessicale" },
        }),
      ],
    });
    expect(scelteIniziali(p)).toEqual([
      { chiave: "ruolo", azione: "crea", titolo: "Ruolo senior engineer" },
      { chiave: "formato", azione: "riusa", titolo: "Formato JSON" },
    ]);
  });
});

describe("conScelta", () => {
  it("restituisce un nuovo array con la sola scelta cambiata", () => {
    const prima = scelteIniziali(proposta());
    const dopo = conScelta(prima, "ruolo", { azione: "scarta" });
    expect(dopo).not.toBe(prima);
    expect(prima[0].azione).toBe("crea");
    expect(dopo[0].azione).toBe("scarta");
  });
});

describe("motiviBloccanti", () => {
  it("è vuoto nel caso felice", () => {
    const p = proposta();
    expect(motiviBloccanti(p, scelteIniziali(p))).toEqual([]);
  });

  it("blocca titolo vuoto, barra, doppioni e conflitto col vault non risolto", () => {
    const p = proposta({
      moduli: [
        modulo({ chiave: "a", titolo: "Ruolo A", titolo_in_conflitto: "esistente" }),
        modulo({ chiave: "b", titolo: "Ruolo B" }),
        modulo({ chiave: "c", titolo: "Ruolo C" }),
        modulo({ chiave: "d", titolo: "Ruolo D" }),
      ],
    });
    let scelte = scelteIniziali(p);
    scelte = conScelta(scelte, "b", { titolo: "  " });
    scelte = conScelta(scelte, "c", { titolo: "Ruoli/Uno" });
    scelte = conScelta(scelte, "d", { titolo: "ruolo a" });
    const motivi = motiviBloccanti(p, scelte);
    expect(motivi.some((m) => m.includes("Esiste già un prompt intitolato «Ruolo A»"))).toBe(true);
    expect(motivi.some((m) => m.includes("non ha titolo"))).toBe(true);
    expect(motivi.some((m) => m.includes("«/»"))).toBe(true);
    expect(motivi.some((m) => m.includes("stesso titolo"))).toBe(true);
  });

  it("il conflitto col vault sparisce se l'utente rinomina o riusa", () => {
    const p = proposta({
      moduli: [
        modulo({
          titolo_in_conflitto: "esistente",
          riuso: { prompt_id: "esistente", titolo: "Ruolo senior engineer", similarita: 1, fonte: "lessicale" },
        }),
      ],
    });
    expect(motiviBloccanti(p, scelteIniziali(p))).toEqual([]); // riusa
    const rinominato = conScelta(
      [{ chiave: "ruolo", azione: "crea", titolo: "Ruolo senior engineer" }],
      "ruolo",
      { titolo: "Ruolo senior engineer (PaP)" },
    );
    expect(motiviBloccanti(p, rinominato)).toEqual([]);
  });

  it("blocca import non risolti, problemi del linter e proposta senza prompt", () => {
    const p = proposta({
      moduli: [modulo({ problemi: ["PH001: graffa singola"] })],
      prompt: [prompt({ import_non_risolti: ["Sparito"], problemi: ["LEN001: troppo lungo"] })],
    });
    const motivi = motiviBloccanti(p, scelteIniziali(p));
    expect(motivi).toHaveLength(3);
    expect(motiviBloccanti(proposta({ prompt: [] }), [])).toEqual([
      "Il modello non ha restituito nessun prompt ricomposto.",
    ]);
  });

  it("un modulo scartato non porta i suoi problemi", () => {
    const p = proposta({ moduli: [modulo({ problemi: ["PH001: x"] })] });
    const scelte = conScelta(scelteIniziali(p), "ruolo", { azione: "scarta" });
    // Resta solo l'import che ora non risolve? No: la verifica degli import
    // è del backend; qui il prompt ha import_non_risolti vuoto.
    expect(motiviBloccanti(p, scelte)).toEqual([]);
  });
});

describe("riscriviImportRinominati", () => {
  it("segue il nuovo titolo sui token import, case-insensitive, e non tocca la prosa", () => {
    const p = proposta();
    const scelte = conScelta(scelteIniziali(p), "ruolo", { titolo: "Ruolo tech lead" });
    const corpo =
      'Nella prosa: import "Ruolo senior engineer" resta.\n{{import "ruolo SENIOR engineer" with x=1}}\n{{import "Altro"}}';
    expect(riscriviImportRinominati(corpo, p, scelte)).toBe(
      'Nella prosa: import "Ruolo senior engineer" resta.\n{{import "Ruolo tech lead" with x=1}}\n{{import "Altro"}}',
    );
  });

  it("non riscrive se il modulo è riusato o il titolo è invariato", () => {
    const p = proposta();
    const corpo = '{{import "Ruolo senior engineer"}}';
    expect(riscriviImportRinominati(corpo, p, scelteIniziali(p))).toBe(corpo);
    const riusa = conScelta(scelteIniziali(p), "ruolo", { azione: "riusa", titolo: "Altro" });
    expect(riscriviImportRinominati(corpo, p, riusa)).toBe(corpo);
  });
});

describe("costruisciRichiesta", () => {
  it("mappa scelte e riscritture nella forma attesa dal backend", () => {
    const p = proposta({
      moduli: [
        modulo(),
        modulo({
          chiave: "formato",
          titolo: "Formato JSON",
          tipo: "formato",
          corpo: "Rispondi in JSON.",
          riuso: { prompt_id: "x", titolo: "Rispondi in JSON", similarita: 0.9, fonte: "lessicale" },
        }),
        modulo({ chiave: "extra", titolo: "Extra", corpo: "e" }),
      ],
    });
    let scelte = scelteIniziali(p);
    scelte = conScelta(scelte, "ruolo", { titolo: " Ruolo tech lead " });
    scelte = conScelta(scelte, "extra", { azione: "scarta" });

    const r = costruisciRichiesta(p, scelte);

    expect(r.moduli).toEqual([
      { titolo: "Ruolo tech lead", tipo: "ruolo", corpo: "Sei un senior engineer.", azione: { tipo: "crea" } },
      { titolo: "Formato JSON", tipo: "formato", corpo: "Rispondi in JSON.", azione: { tipo: "riusa", prompt_id: "x" } },
      { titolo: "Extra", tipo: "ruolo", corpo: "e", azione: { tipo: "scarta" } },
    ]);
    expect(r.prompt[0].corpo).toBe('{{import "Ruolo tech lead"}}\n{{codice}}');
    expect(r.prompt[0].corpo_originale_atteso).toBe("Sei un senior engineer.\n{{codice}}");
  });

  it("di default conserva il titolo originale del prompt, non quello del modello", () => {
    const p = proposta(); // titolo proposto «Code review», originale «Review»
    const r = costruisciRichiesta(p, scelteIniziali(p));
    expect(r.prompt[0].titolo).toBe("Review");
    expect("descrizione" in r.prompt[0]).toBe(false);
  });

  it("usa il titolo scelto dall'utente se lo cambia", () => {
    const p = proposta();
    const sp = conTitoloPrompt(scelteInizialiPrompt(p), "p1", " Code review PaP ");
    const r = costruisciRichiesta(p, scelteIniziali(p), sp);
    expect(r.prompt[0].titolo).toBe("Code review PaP");
    expect(motiviBloccanti(p, scelteIniziali(p), conTitoloPrompt(sp, "p1", "  "))).toEqual([
      "Il prompt «Review» non può restare senza titolo.",
    ]);
  });
});

describe("formattaEsito", () => {
  it("riassume moduli e prompt con la cartella", () => {
    expect(
      formattaEsito({
        moduli_creati: [{ id: "a", titolo: "A" }, { id: "b", titolo: "B" }],
        moduli_riusati: [{ id: "c", titolo: "C" }],
        prompt_aggiornati: [{ id: "p", titolo: "P" }],
        cartella_moduli: "/Sviluppo/Moduli",
      }),
    ).toBe("2 moduli creati · 1 modulo riusato · 1 prompt ricomposto (nuova versione) · moduli in /Sviluppo/Moduli");
  });
});
