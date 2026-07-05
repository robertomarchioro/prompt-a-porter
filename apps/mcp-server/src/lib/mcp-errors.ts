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
