import { describe, it, expect } from "vitest";
import {
  MINIMO_PER_PROVIDER,
  ammesso,
  costruisciRegistro,
  descriviDelta,
  estraiModelli,
  fondi,
  verificaSanita,
  type ListinoGrezzo,
  type VoceListino,
} from "./modelli-aggiornamento";
import type { ModelloAI, RegistroModelli } from "./modelli-provider";

const OGGI = "2026-08-01";

const chat = (p: Partial<VoceListino> = {}): VoceListino => ({
  name: "Modello",
  release_date: "2026-01-01",
  modalities: { input: ["text"], output: ["text"] },
  limit: { context: 200_000 },
  cost: { input: 1, output: 5 },
  ...p,
});

const modello = (p: Partial<ModelloAI>): ModelloAI => ({
  id: "x",
  provider: "anthropic",
  etichetta: "X",
  obsoleto: false,
  anteprima: false,
  visto_il: "2026-01-01",
  contesto: null,
  prezzo: null,
  ...p,
});

describe("ammesso — filtro del listino", () => {
  it("accetta un normale modello di chat testuale", () => {
    expect(ammesso("claude-opus-5", chat(), "anthropic")).toBe(true);
  });

  it("scarta i modelli che producono immagini o audio", () => {
    const img = chat({ modalities: { input: ["text"], output: ["image"] } });
    expect(ammesso("gemini-3-pro-image", img, "gemini")).toBe(false);
  });

  it("scarta gli embedding, che dichiarano output testuale ma costo output 0", () => {
    const emb = chat({ cost: { input: 0.13, output: 0 } });
    expect(ammesso("text-embedding-3-large", emb, "openai")).toBe(false);
  });

  it("scarta gli alias datati e tiene l'id stabile", () => {
    expect(ammesso("gpt-4o-2024-05-13", chat(), "openai")).toBe(false);
    expect(ammesso("gpt-4o", chat(), "openai")).toBe(true);
  });

  it("scarta i modelli a pesi aperti", () => {
    expect(ammesso("gemma-4-31b-it", chat({ open_weights: true }), "gemini")).toBe(false);
  });

  it("scarta i modelli più vecchi della soglia di rilascio", () => {
    expect(ammesso("gpt-4", chat({ release_date: "2023-11-06" }), "openai")).toBe(false);
  });

  it("scarta le varianti specialistiche di OpenAI", () => {
    for (const id of ["gpt-5.5-pro", "gpt-5.3-codex", "o1-pro", "o3", "gpt-5.2-chat-latest"]) {
      expect(ammesso(id, chat(), "openai")).toBe(false);
    }
  });

  it("le stesse forme NON sono filtrate sugli altri provider", () => {
    // `-pro` è una variante specialistica solo in casa OpenAI: per Gemini è
    // la linea di punta.
    expect(ammesso("gemini-3.1-pro-preview", chat(), "gemini")).toBe(true);
  });

  it("ammette la sola anteprima Pro di Gemini, non le anteprime di nicchia", () => {
    expect(ammesso("gemini-3-pro-preview", chat(), "gemini")).toBe(true);
    expect(ammesso("gemini-3.1-pro-preview-customtools", chat(), "gemini")).toBe(false);
    expect(ammesso("gemini-robotics-er-1.6-preview", chat(), "gemini")).toBe(false);
    expect(ammesso("gemini-2.5-computer-use-preview-10-2025", chat(), "gemini")).toBe(
      false,
    );
  });
});

describe("estraiModelli", () => {
  const listino: ListinoGrezzo = {
    anthropic: { models: { "claude-opus-5": chat({ name: "Claude Opus 5" }) } },
    google: { models: { "gemini-3.6-flash": chat({ name: "Gemini 3.6 Flash" }) } },
    openai: { models: { "gpt-5.6": chat({ name: "GPT-5.6" }) } },
    // Provider non previsti da PAP: ignorati.
    mistral: { models: { "mistral-large": chat() } },
  };

  it("mappa google → gemini e ignora i provider non previsti", () => {
    const m = estraiModelli(listino, OGGI);
    expect(m.map((x) => x.provider).sort()).toEqual(["anthropic", "gemini", "openai"]);
  });

  it("riporta etichetta, contesto, prezzo e data di avvistamento", () => {
    const m = estraiModelli(listino, OGGI).find((x) => x.id === "claude-opus-5");
    expect(m).toMatchObject({
      etichetta: "Claude Opus 5",
      contesto: 200_000,
      prezzo: { input: 1, output: 5 },
      visto_il: OGGI,
      obsoleto: false,
    });
  });

  it("gestisce un provider senza modelli senza esplodere", () => {
    expect(estraiModelli({ anthropic: {} }, OGGI)).toEqual([]);
  });
});

describe("fondi — regola degli obsoleti", () => {
  it("marca obsoleto ciò che sparisce dal listino, senza rimuoverlo", () => {
    const corrente = [modello({ id: "vecchio" }), modello({ id: "ancora" })];
    const listino = [modello({ id: "ancora", visto_il: OGGI })];

    const out = fondi(corrente, listino, OGGI);

    expect(out.map((m) => m.id).sort()).toEqual(["ancora", "vecchio"]);
    expect(out.find((m) => m.id === "vecchio")?.obsoleto).toBe(true);
    expect(out.find((m) => m.id === "ancora")?.obsoleto).toBe(false);
  });

  it("un modello che ricompare torna attivo", () => {
    const corrente = [modello({ id: "tornato", obsoleto: true })];
    const out = fondi(corrente, [modello({ id: "tornato" })], OGGI);
    expect(out[0].obsoleto).toBe(false);
    expect(out[0].visto_il).toBe(OGGI);
  });

  it("aggiunge i modelli nuovi", () => {
    const out = fondi([], [modello({ id: "nuovo" })], OGGI);
    expect(out).toHaveLength(1);
    expect(out[0].id).toBe("nuovo");
  });

  it("non muta gli oggetti in ingresso", () => {
    const corrente = [modello({ id: "vecchio" })];
    const copia = structuredClone(corrente);
    fondi(corrente, [], OGGI);
    expect(corrente).toEqual(copia);
  });

  it("aggiorna i metadati dal listino (prezzo cambiato)", () => {
    const corrente = [modello({ id: "a", prezzo: { input: 1, output: 5 } })];
    const listino = [modello({ id: "a", prezzo: { input: 2, output: 10 } })];
    expect(fondi(corrente, listino, OGGI)[0].prezzo).toEqual({ input: 2, output: 10 });
  });
});

describe("descriviDelta", () => {
  it("distingue nuovi, obsoleti e tornati", () => {
    const prima = [
      modello({ id: "resta" }),
      modello({ id: "sparisce" }),
      modello({ id: "torna", obsoleto: true }),
    ];
    const dopo = [
      modello({ id: "resta" }),
      modello({ id: "sparisce", obsoleto: true }),
      modello({ id: "torna" }),
      modello({ id: "arriva" }),
    ];

    expect(descriviDelta(prima, dopo)).toEqual({
      nuovi: ["arriva"],
      obsoleti: ["sparisce"],
      tornati: ["torna"],
    });
  });

  it("nessun cambiamento → tutte le liste vuote", () => {
    const stesso = [modello({ id: "a" })];
    expect(descriviDelta(stesso, stesso)).toEqual({
      nuovi: [],
      obsoleti: [],
      tornati: [],
    });
  });
});

describe("verificaSanita — il listino rotto non deve passare", () => {
  it("blocca un listino vuoto", () => {
    expect(verificaSanita([])).toMatch(/Listino sospetto/);
  });

  it("blocca quando un solo provider è sotto la soglia", () => {
    const pieno = (p: ModelloAI["provider"]) =>
      Array.from({ length: MINIMO_PER_PROVIDER }, (_, i) =>
        modello({ id: `${p}-${i}`, provider: p }),
      );
    const listino = [...pieno("anthropic"), ...pieno("openai"), modello({ provider: "gemini" })];
    expect(verificaSanita(listino)).toMatch(/gemini/);
  });

  it("passa quando tutti i provider sono coperti", () => {
    const pieno = (p: ModelloAI["provider"]) =>
      Array.from({ length: MINIMO_PER_PROVIDER }, (_, i) =>
        modello({ id: `${p}-${i}`, provider: p }),
      );
    expect(verificaSanita([...pieno("anthropic"), ...pieno("openai"), ...pieno("gemini")])).toBe(
      null,
    );
  });
});

describe("costruisciRegistro", () => {
  const corrente: RegistroModelli = {
    aggiornato_a: "2026-07-01",
    fonte: "https://models.dev/api.json",
    modelli: [modello({ id: "claude-opus-5" })],
  };

  const listinoBuono: ListinoGrezzo = {
    anthropic: {
      models: Object.fromEntries(
        ["claude-opus-5", "claude-sonnet-5", "claude-haiku-4-5"].map((id) => [id, chat()]),
      ),
    },
    openai: {
      models: Object.fromEntries(
        ["gpt-5.6", "gpt-5.5", "gpt-5.4"].map((id) => [id, chat()]),
      ),
    },
    google: {
      models: Object.fromEntries(
        ["gemini-3.6-flash", "gemini-3.5-flash", "gemini-2.5-pro"].map((id) => [id, chat()]),
      ),
    },
  };

  it("aggiorna registro e data quando il listino è sano", () => {
    const { registro, errore } = costruisciRegistro(corrente, listinoBuono, OGGI);
    expect(errore).toBe(null);
    expect(registro.aggiornato_a).toBe(OGGI);
    expect(registro.modelli).toHaveLength(9);
  });

  it("un listino rotto NON scrive nulla e restituisce l'errore", () => {
    const { registro, errore } = costruisciRegistro(corrente, { anthropic: {} }, OGGI);
    expect(errore).toMatch(/Listino sospetto/);
    // Il registro torna identico: nessun modello marcato obsoleto per sbaglio.
    expect(registro).toBe(corrente);
    expect(registro.modelli[0].obsoleto).toBe(false);
  });
});
