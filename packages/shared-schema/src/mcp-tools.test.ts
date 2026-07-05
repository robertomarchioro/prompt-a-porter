import { describe, it, expect } from "vitest";
import {
  MAX_TAGS_COUNT,
  MAX_TEXT_LENGTH,
  papGetArgsSchema,
  papListRecentArgsSchema,
  papRenderArgsSchema,
  papSearchArgsSchema,
} from "./mcp-tools.js";

describe("papSearchArgsSchema", () => {
  it("accetta argomenti validi completi", () => {
    const risultato = papSearchArgsSchema.safeParse({
      query: "hello",
      limit: 10,
      target_model: "claude-sonnet",
      tags: ["a", "b"],
    });
    expect(risultato.success).toBe(true);
  });

  it("accetta oggetto vuoto (tutti i campi sono opzionali)", () => {
    expect(papSearchArgsSchema.safeParse({}).success).toBe(true);
  });

  it("rifiuta query di tipo sbagliato", () => {
    const risultato = papSearchArgsSchema.safeParse({ query: 123 });
    expect(risultato.success).toBe(false);
  });

  it("rifiuta limit di tipo sbagliato", () => {
    const risultato = papSearchArgsSchema.safeParse({ limit: "10" });
    expect(risultato.success).toBe(false);
  });

  it("rifiuta limit non intero", () => {
    const risultato = papSearchArgsSchema.safeParse({ limit: 1.5 });
    expect(risultato.success).toBe(false);
  });

  it("accetta limit fuori range (0, negativo, >50): il clamp avviene lato server", () => {
    // Il vecchio comportamento faceva clamp silenzioso in promptCerca/
    // promptListRecent (Math.min(Math.max(limit,1),50)); lo schema valida
    // solo il tipo (intero) per non trasformare un caso prima "tollerato"
    // in un hard-fail regressivo.
    expect(papSearchArgsSchema.safeParse({ limit: 51 }).success).toBe(true);
    expect(papSearchArgsSchema.safeParse({ limit: 0 }).success).toBe(true);
    expect(papSearchArgsSchema.safeParse({ limit: -1 }).success).toBe(true);
  });

  it("rifiuta query oversize oltre MAX_TEXT_LENGTH", () => {
    const query = "a".repeat(MAX_TEXT_LENGTH + 1);
    const risultato = papSearchArgsSchema.safeParse({ query });
    expect(risultato.success).toBe(false);
  });

  it("accetta query esattamente a MAX_TEXT_LENGTH", () => {
    const query = "a".repeat(MAX_TEXT_LENGTH);
    expect(papSearchArgsSchema.safeParse({ query }).success).toBe(true);
  });

  it("rifiuta array tags oltre MAX_TAGS_COUNT", () => {
    const tags = Array.from({ length: MAX_TAGS_COUNT + 1 }, (_, i) => `tag-${i}`);
    const risultato = papSearchArgsSchema.safeParse({ tags });
    expect(risultato.success).toBe(false);
  });

  it("accetta array tags esattamente a MAX_TAGS_COUNT", () => {
    const tags = Array.from({ length: MAX_TAGS_COUNT }, (_, i) => `tag-${i}`);
    expect(papSearchArgsSchema.safeParse({ tags }).success).toBe(true);
  });

  it("rifiuta tags con elementi non stringa", () => {
    const risultato = papSearchArgsSchema.safeParse({ tags: ["ok", 42] });
    expect(risultato.success).toBe(false);
  });

  it("rifiuta campi extra non dichiarati (strict)", () => {
    const risultato = papSearchArgsSchema.safeParse({ query: "x", extra: "nope" });
    expect(risultato.success).toBe(false);
  });

  it("accetta null esplicito per query/limit/target_model/tags (client che serializzano unset come null)", () => {
    const risultato = papSearchArgsSchema.safeParse({
      query: null,
      limit: null,
      target_model: null,
      tags: null,
    });
    expect(risultato.success).toBe(true);
    if (risultato.success) {
      expect(risultato.data.query).toBeNull();
      expect(risultato.data.limit).toBeNull();
      expect(risultato.data.target_model).toBeNull();
      expect(risultato.data.tags).toBeNull();
    }
  });

  it("null equivale a undefined per il pattern `a.x ?? default` usato nel server", () => {
    const risultato = papSearchArgsSchema.safeParse({
      query: null,
      limit: null,
      target_model: null,
      tags: null,
    });
    expect(risultato.success).toBe(true);
    if (!risultato.success) return;
    const a = risultato.data;
    expect(a.query ?? "").toBe("");
    expect(a.limit ?? 10).toBe(10);
    expect(a.target_model ?? null).toBeNull();
    expect(a.tags ?? []).toEqual([]);
  });
});

describe("papGetArgsSchema", () => {
  it("accetta prompt_id valido", () => {
    expect(papGetArgsSchema.safeParse({ prompt_id: "abc-123" }).success).toBe(true);
  });

  it("rifiuta senza prompt_id (campo obbligatorio)", () => {
    expect(papGetArgsSchema.safeParse({}).success).toBe(false);
  });

  it("rifiuta prompt_id vuoto", () => {
    expect(papGetArgsSchema.safeParse({ prompt_id: "" }).success).toBe(false);
  });

  it("rifiuta prompt_id di tipo sbagliato", () => {
    expect(papGetArgsSchema.safeParse({ prompt_id: 123 }).success).toBe(false);
  });
});

describe("papListRecentArgsSchema", () => {
  it("accetta oggetto vuoto", () => {
    expect(papListRecentArgsSchema.safeParse({}).success).toBe(true);
  });

  it("accetta limit negativo (il clamp avviene lato server, non nello schema)", () => {
    expect(papListRecentArgsSchema.safeParse({ limit: -1 }).success).toBe(true);
  });

  it("rifiuta limit di tipo sbagliato", () => {
    expect(papListRecentArgsSchema.safeParse({ limit: "10" }).success).toBe(false);
  });

  it("accetta limit null", () => {
    const risultato = papListRecentArgsSchema.safeParse({ limit: null });
    expect(risultato.success).toBe(true);
    if (risultato.success) expect(risultato.data.limit).toBeNull();
  });
});

describe("papRenderArgsSchema", () => {
  it("accetta prompt_id + vars validi", () => {
    const risultato = papRenderArgsSchema.safeParse({
      prompt_id: "abc",
      vars: { nome: "Mario", ruolo: "sviluppatore" },
    });
    expect(risultato.success).toBe(true);
  });

  it("accetta senza vars (opzionale)", () => {
    expect(papRenderArgsSchema.safeParse({ prompt_id: "abc" }).success).toBe(true);
  });

  it("rifiuta vars con valore non stringa", () => {
    const risultato = papRenderArgsSchema.safeParse({
      prompt_id: "abc",
      vars: { nome: 42 },
    });
    expect(risultato.success).toBe(false);
  });

  it("rifiuta valore vars oversize oltre MAX_TEXT_LENGTH", () => {
    const risultato = papRenderArgsSchema.safeParse({
      prompt_id: "abc",
      vars: { nome: "a".repeat(MAX_TEXT_LENGTH + 1) },
    });
    expect(risultato.success).toBe(false);
  });

  it("rifiuta troppi segnaposti in vars", () => {
    const vars: Record<string, string> = {};
    for (let i = 0; i < 201; i++) vars[`k${i}`] = "v";
    const risultato = papRenderArgsSchema.safeParse({ prompt_id: "abc", vars });
    expect(risultato.success).toBe(false);
  });

  it("accetta vars null (client che serializzano unset come null) e coalesce al default", () => {
    const risultato = papRenderArgsSchema.safeParse({ prompt_id: "abc", vars: null });
    expect(risultato.success).toBe(true);
    if (!risultato.success) return;
    expect(risultato.data.vars).toBeNull();
    expect(risultato.data.vars ?? {}).toEqual({});
  });
});
