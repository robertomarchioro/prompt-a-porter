import { describe, it, expect } from "vitest";
import { z } from "zod";
import { rispostaErroreValidazione } from "./mcp-errors.js";

function erroreDa(schema: z.ZodType, dato: unknown): z.ZodError {
  const risultato = schema.safeParse(dato);
  if (risultato.success) {
    throw new Error("Atteso un errore di validazione ma il parse è riuscito");
  }
  return risultato.error;
}

describe("rispostaErroreValidazione", () => {
  it("segnala isError true e include il nome del tool nel messaggio", () => {
    const schema = z.object({ prompt_id: z.string() });
    const error = erroreDa(schema, {});

    const risposta = rispostaErroreValidazione("pap_get", error);

    expect(risposta.isError).toBe(true);
    expect(risposta.content[0].text).toContain("pap_get");
  });

  it("elenca il path del campo non valido nel messaggio", () => {
    const schema = z.object({ vars: z.record(z.string(), z.string()) });
    const error = erroreDa(schema, { vars: { nome: 42 } });

    const risposta = rispostaErroreValidazione("pap_render", error);

    expect(risposta.content[0].text).toContain("vars.nome");
  });

  it("usa '(radice)' quando l'errore non ha un path", () => {
    const schema = z.string();
    const error = erroreDa(schema, 42);

    const risposta = rispostaErroreValidazione("pap_search", error);

    expect(risposta.content[0].text).toContain("(radice)");
  });

  it("concatena più errori con '; '", () => {
    const schema = z.object({ a: z.string(), b: z.number() });
    const error = erroreDa(schema, { a: 1, b: "x" });

    const risposta = rispostaErroreValidazione("pap_search", error);

    expect(risposta.content[0].text.split("; ")).toHaveLength(2);
  });
});
