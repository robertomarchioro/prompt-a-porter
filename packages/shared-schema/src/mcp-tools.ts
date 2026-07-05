/**
 * Schema Zod per gli argomenti dei tool MCP di Prompt a Porter.
 *
 * Il SDK MCP valida solo la busta `{ name, arguments }` della richiesta,
 * non la forma dichiarata in `inputSchema` di ogni tool. Questi schema
 * forniscono la validazione runtime mancante lato server (tipo, presenza
 * dei campi obbligatori) più limiti dimensionali contro payload
 * eccessivamente grandi (stringhe lunghissime, array enormi).
 */
import { z } from "zod";

/** Lunghezza massima per una singola stringa "libera" (query, valore vars). */
export const MAX_TEXT_LENGTH = 8000;

/** Lunghezza massima per identificatori/stringhe corte (prompt_id, target_model, tag). */
export const MAX_SHORT_STRING_LENGTH = 200;

/** Numero massimo di tag ammessi in un filtro di ricerca. */
export const MAX_TAGS_COUNT = 50;

/** Numero massimo di segnaposti ammessi in una richiesta di render. */
export const MAX_VARS_COUNT = 200;

const promptIdSchema = z
  .string({ error: "prompt_id deve essere una stringa" })
  .min(1, "prompt_id non può essere vuoto")
  .max(MAX_SHORT_STRING_LENGTH, `prompt_id supera i ${MAX_SHORT_STRING_LENGTH} caratteri`);

// `limit` viene comunque bloccato tra 1 e 50 lato server (vedi promptCerca/
// promptListRecent in index.ts): qui validiamo solo il tipo (numero intero),
// non l'intervallo, per preservare il comportamento storico di "clamp"
// invece di un hard-fail su valori fuori range (es. 0, negativi, >50).
const limitSchema = z
  .number({ error: "limit deve essere un numero" })
  .int("limit deve essere un intero")
  .nullish();

// `.nullish()` (accetta anche `null`, non solo `undefined`) perché molti
// client MCP serializzano un argomento opzionale non impostato come
// `null` esplicito nel JSON invece di ometterlo del tutto.
const tagsSchema = z
  .array(
    z
      .string({ error: "ogni tag deve essere una stringa" })
      .max(MAX_SHORT_STRING_LENGTH, `tag oltre i ${MAX_SHORT_STRING_LENGTH} caratteri`),
    { error: "tags deve essere un array di stringhe" },
  )
  .max(MAX_TAGS_COUNT, `troppi tag (max ${MAX_TAGS_COUNT})`)
  .nullish();

export const papSearchArgsSchema = z
  .object({
    query: z
      .string({ error: "query deve essere una stringa" })
      .max(MAX_TEXT_LENGTH, `query oltre i ${MAX_TEXT_LENGTH} caratteri`)
      .nullish(),
    limit: limitSchema,
    target_model: z
      .string({ error: "target_model deve essere una stringa" })
      .max(MAX_SHORT_STRING_LENGTH, `target_model oltre i ${MAX_SHORT_STRING_LENGTH} caratteri`)
      .nullish(),
    tags: tagsSchema,
  })
  .strict();

export type PapSearchArgs = z.infer<typeof papSearchArgsSchema>;

export const papGetArgsSchema = z
  .object({
    prompt_id: promptIdSchema,
  })
  .strict();

export type PapGetArgs = z.infer<typeof papGetArgsSchema>;

export const papListRecentArgsSchema = z
  .object({
    limit: limitSchema,
  })
  .strict();

export type PapListRecentArgs = z.infer<typeof papListRecentArgsSchema>;

export const papRenderArgsSchema = z
  .object({
    prompt_id: promptIdSchema,
    vars: z
      .record(
        z.string(),
        z
          .string({ error: "ogni valore di vars deve essere una stringa" })
          .max(MAX_TEXT_LENGTH, `valore di vars oltre i ${MAX_TEXT_LENGTH} caratteri`),
        { error: "vars deve essere un oggetto nome_segnaposto -> valore" },
      )
      .refine((v) => Object.keys(v).length <= MAX_VARS_COUNT, {
        message: `troppi segnaposti in vars (max ${MAX_VARS_COUNT})`,
      })
      .nullish(),
  })
  .strict();

export type PapRenderArgs = z.infer<typeof papRenderArgsSchema>;

/** Mappa nome-tool -> schema Zod dei suoi argomenti, usata dal server MCP. */
export const TOOL_ARGS_SCHEMAS = {
  pap_search: papSearchArgsSchema,
  pap_get: papGetArgsSchema,
  pap_list_recent: papListRecentArgsSchema,
  pap_render: papRenderArgsSchema,
} as const;

export type ToolName = keyof typeof TOOL_ARGS_SCHEMAS;
