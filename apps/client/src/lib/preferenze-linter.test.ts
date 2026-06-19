// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from "vitest";
import {
  CATEGORIE_LINTER,
  ETICHETTE,
  DESCRIZIONI,
  leggiCategorieDisabilitate,
  salvaCategorieDisabilitate,
  categoriaAttiva,
  toggleCategoria,
  leggiRegoleDisabilitate,
  salvaRegoleDisabilitate,
  toggleRegola,
  DEFAULT_SOGLIE,
  leggiConfig,
  salvaConfig,
  setSeverita,
  setSoglia,
  type ConfigLinter,
} from "./preferenze-linter";

const KEY = "pap.linter.categorie_disabilitate";
const KEY_REGOLE = "pap.linter.regole_disabilitate";
const KEY_CONFIG = "pap.linter.config";

describe("preferenze-linter", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("CATEGORIE_LINTER ha 5 elementi e match con ETICHETTE/DESCRIZIONI", () => {
    expect(CATEGORIE_LINTER).toHaveLength(5);
    for (const c of CATEGORIE_LINTER) {
      expect(ETICHETTE[c]).toBeTruthy();
      expect(DESCRIZIONI[c]).toBeTruthy();
    }
  });

  it("leggiCategorieDisabilitate ritorna [] se localStorage vuoto", () => {
    expect(leggiCategorieDisabilitate()).toEqual([]);
  });

  it("leggiCategorieDisabilitate filtra valori non validi", () => {
    localStorage.setItem(KEY, JSON.stringify(["LEN", "FAKE", "PH", 42]));
    expect(leggiCategorieDisabilitate()).toEqual(["LEN", "PH"]);
  });

  it("leggiCategorieDisabilitate ritorna [] su JSON malformato", () => {
    localStorage.setItem(KEY, "not-json {{");
    expect(leggiCategorieDisabilitate()).toEqual([]);
  });

  it("leggiCategorieDisabilitate ritorna [] se valore non e array", () => {
    localStorage.setItem(KEY, JSON.stringify({ a: 1 }));
    expect(leggiCategorieDisabilitate()).toEqual([]);
  });

  it("salvaCategorieDisabilitate scrive solo valori validi", () => {
    salvaCategorieDisabilitate(["LEN", "PH", "X" as unknown as "LEN"]);
    const raw = localStorage.getItem(KEY);
    expect(JSON.parse(raw!)).toEqual(["LEN", "PH"]);
  });

  it("categoriaAttiva true quando non disabilitata", () => {
    expect(categoriaAttiva("LEN", [])).toBe(true);
    expect(categoriaAttiva("LEN", ["PH"])).toBe(true);
  });

  it("categoriaAttiva false quando disabilitata", () => {
    expect(categoriaAttiva("PH", ["PH", "PII"])).toBe(false);
  });

  it("toggleCategoria aggiunge se assente", () => {
    expect(toggleCategoria("LEN", [])).toEqual(["LEN"]);
    expect(toggleCategoria("PH", ["LEN"])).toEqual(["LEN", "PH"]);
  });

  it("toggleCategoria rimuove se presente e non muta input", () => {
    const input = ["LEN", "PH"] as const;
    const out = toggleCategoria("LEN", input);
    expect(out).toEqual(["PH"]);
    expect(input).toEqual(["LEN", "PH"]);
  });

  it("round trip salva->leggi", () => {
    salvaCategorieDisabilitate(["STY", "IMP"]);
    expect(leggiCategorieDisabilitate()).toEqual(["STY", "IMP"]);
  });
});

describe("preferenze-linter — per-regola (Fase 1)", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("leggiRegoleDisabilitate ritorna [] se tutto vuoto", () => {
    expect(leggiRegoleDisabilitate()).toEqual([]);
  });

  it("migra one-shot dalla vecchia key categorie se la nuova è assente", () => {
    localStorage.setItem(KEY, JSON.stringify(["PII", "IMP"]));
    expect(leggiRegoleDisabilitate()).toEqual(["PII", "IMP"]);
    // La migrazione scrive anche la nuova key.
    expect(JSON.parse(localStorage.getItem(KEY_REGOLE)!)).toEqual(["PII", "IMP"]);
  });

  it("la nuova key ha precedenza sulla vecchia (niente migrazione)", () => {
    localStorage.setItem(KEY, JSON.stringify(["PII"]));
    localStorage.setItem(KEY_REGOLE, JSON.stringify(["LEN001"]));
    expect(leggiRegoleDisabilitate()).toEqual(["LEN001"]);
  });

  it("accetta code completi e prefissi, filtra non-stringhe", () => {
    localStorage.setItem(KEY_REGOLE, JSON.stringify(["PII001", "LEN", 7, ""]));
    expect(leggiRegoleDisabilitate()).toEqual(["PII001", "LEN"]);
  });

  it("leggiRegoleDisabilitate ritorna [] su JSON malformato", () => {
    localStorage.setItem(KEY_REGOLE, "{{ not json");
    expect(leggiRegoleDisabilitate()).toEqual([]);
  });

  it("salvaRegoleDisabilitate scrive solo stringhe non vuote", () => {
    salvaRegoleDisabilitate(["PII001", "", "STY"]);
    expect(JSON.parse(localStorage.getItem(KEY_REGOLE)!)).toEqual([
      "PII001",
      "STY",
    ]);
  });

  it("toggleRegola aggiunge/rimuove un code e non muta l'input", () => {
    expect(toggleRegola("PII001", [])).toEqual(["PII001"]);
    const input = ["PII001", "LEN"] as const;
    const out = toggleRegola("PII001", input);
    expect(out).toEqual(["LEN"]);
    expect(input).toEqual(["PII001", "LEN"]);
  });
});

describe("preferenze-linter — config completa (Fase 2)", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("leggiConfig ritorna default se tutto vuoto", () => {
    const cfg = leggiConfig();
    expect(cfg.disabilitate).toEqual([]);
    expect(cfg.severita_override).toEqual({});
    expect(cfg.soglie).toEqual(DEFAULT_SOGLIE);
  });

  it("migra disabilitate dalla key Fase 1 se config assente", () => {
    localStorage.setItem(KEY_REGOLE, JSON.stringify(["PII001", "LEN"]));
    const cfg = leggiConfig();
    expect(cfg.disabilitate).toEqual(["PII001", "LEN"]);
    // La migrazione persiste il nuovo blob.
    expect(JSON.parse(localStorage.getItem(KEY_CONFIG)!).disabilitate).toEqual([
      "PII001",
      "LEN",
    ]);
  });

  it("la key config ha precedenza sulla key Fase 1", () => {
    localStorage.setItem(KEY_REGOLE, JSON.stringify(["PII"]));
    localStorage.setItem(
      KEY_CONFIG,
      JSON.stringify({ disabilitate: ["STY001"] }),
    );
    expect(leggiConfig().disabilitate).toEqual(["STY001"]);
  });

  it("normalizza override invalidi e soglie non numeriche", () => {
    localStorage.setItem(
      KEY_CONFIG,
      JSON.stringify({
        disabilitate: ["", "PII001", 5],
        severita_override: { PII001: "error", BAD: "boom", "": "info" },
        soglie: { len_max_body: "x", len_min_body: 10, ngram_threshold: -3 },
      }),
    );
    const cfg = leggiConfig();
    expect(cfg.disabilitate).toEqual(["PII001"]);
    expect(cfg.severita_override).toEqual({ PII001: "error" });
    expect(cfg.soglie.len_max_body).toBe(DEFAULT_SOGLIE.len_max_body); // "x" → default
    expect(cfg.soglie.len_min_body).toBe(10);
    expect(cfg.soglie.ngram_threshold).toBe(DEFAULT_SOGLIE.ngram_threshold); // -3 → default
  });

  it("leggiConfig ritorna default su JSON malformato", () => {
    localStorage.setItem(KEY_CONFIG, "{{ not json");
    expect(leggiConfig().soglie).toEqual(DEFAULT_SOGLIE);
  });

  it("round trip salva->leggi", () => {
    const cfg: ConfigLinter = {
      disabilitate: ["IMP"],
      severita_override: { LEN001: "info" },
      soglie: { len_max_body: 2000, len_min_body: 10, ngram_threshold: 6 },
    };
    salvaConfig(cfg);
    expect(leggiConfig()).toEqual(cfg);
  });

  it("setSeverita imposta un override e non muta l'input", () => {
    const cfg = leggiConfig();
    const out = setSeverita(cfg, "PII001", "error", "warning");
    expect(out.severita_override).toEqual({ PII001: "error" });
    expect(cfg.severita_override).toEqual({});
  });

  it("setSeverita rimuove l'override se uguale al default", () => {
    const cfg: ConfigLinter = {
      disabilitate: [],
      severita_override: { PII001: "error" },
      soglie: { ...DEFAULT_SOGLIE },
    };
    const out = setSeverita(cfg, "PII001", "warning", "warning");
    expect(out.severita_override).toEqual({});
  });

  it("setSoglia aggiorna un campo e clampa input invalido al default", () => {
    const cfg = leggiConfig();
    const out = setSoglia(cfg, "len_max_body", 1234);
    expect(out.soglie.len_max_body).toBe(1234);
    const bad = setSoglia(cfg, "ngram_threshold", NaN);
    expect(bad.soglie.ngram_threshold).toBe(DEFAULT_SOGLIE.ngram_threshold);
    // immutabilità
    expect(cfg.soglie.len_max_body).toBe(DEFAULT_SOGLIE.len_max_body);
  });

  it("setSoglia applica il minimo per-campo (no divergenza col backend)", () => {
    const cfg = leggiConfig();
    // ngram_threshold ha minimo 2: uno 0 dell'utente viene portato a 2.
    expect(setSoglia(cfg, "ngram_threshold", 0, 2).soglie.ngram_threshold).toBe(
      2,
    );
    // len_max_body minimo 1.
    expect(setSoglia(cfg, "len_max_body", 0, 1).soglie.len_max_body).toBe(1);
    // valore valido sopra il minimo resta invariato.
    expect(setSoglia(cfg, "ngram_threshold", 5, 2).soglie.ngram_threshold).toBe(
      5,
    );
    // valore frazionario viene troncato.
    expect(setSoglia(cfg, "len_min_body", 12.9, 0).soglie.len_min_body).toBe(12);
  });
});
