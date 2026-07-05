/**
 * Formattazione delle risposte di errore MCP per gli handler dei tool.
 */
import type { ZodError } from "zod";

interface RispostaErroreMcp {
  content: [{ type: "text"; text: string }];
  isError: true;
}

/**
 * Formatta un errore di validazione Zod come risposta MCP strutturata
 * (`isError: true`), invece di lasciar propagare un TypeError generico
 * dal cast non verificato degli argomenti.
 */
export function rispostaErroreValidazione(
  toolName: string,
  error: ZodError,
): RispostaErroreMcp {
  const dettagli = error.issues
    .map((issue) => `${issue.path.length ? issue.path.join(".") : "(radice)"}: ${issue.message}`)
    .join("; ");
  return {
    content: [
      {
        type: "text",
        text: `Argomenti non validi per il tool "${toolName}": ${dettagli}`,
      },
    ],
    isError: true,
  };
}

/**
 * Risposta MCP strutturata quando gli argomenti grezzi superano la
 * guardia aggregata di dimensione (vedi `argsTroppoGrandi` in `limits.ts`),
 * PRIMA di essere passati alla validazione Zod per-tool. Evita di far
 * attraversare al parser Zod payload enormi (es. milioni di chiavi) solo
 * per scoprire alla fine che superano `.max()`/`.refine()`.
 */
export function rispostaErroreArgomentiTroppoGrandi(
  toolName: string,
  maxJsonLength: number,
): RispostaErroreMcp {
  return {
    content: [
      {
        type: "text",
        text: `Argomenti per il tool "${toolName}" troppo grandi (oltre ${maxJsonLength} caratteri JSON)`,
      },
    ],
    isError: true,
  };
}
