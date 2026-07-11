#!/usr/bin/env node
/**
 * Prompt a Porter — MCP Server
 *
 * Espone il vault PaP via Model Context Protocol (stdio) per Claude Desktop,
 * Cursor e altri agenti. Read-only per ora; create/update arriveranno con
 * coordinamento del client desktop in step successivi.
 *
 * Vault discovery:
 * - env PAP_VAULT_PATH (override esplicito)
 * - default per piattaforma: <app_data_dir>/com.pap.client/pap-vault.db
 *
 * Limitazioni MVP:
 * - solo vault NON cifrati (better-sqlite3 standard, no SQLCipher)
 * - per vault cifrati serve `better-sqlite3-multiple-ciphers` + password (futuro)
 */

import Database from "better-sqlite3";
import { homedir, platform } from "node:os";
import { join } from "node:path";
import { existsSync } from "node:fs";

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";

import {
  papGetArgsSchema,
  papListRecentArgsSchema,
  papRenderArgsSchema,
  papSearchArgsSchema,
} from "@pap/shared-schema";

import { sanitizzaFts } from "./lib/fts.js";
import { argsTroppoGrandi, clampLimit, MAX_ARGS_JSON_LENGTH } from "./lib/limits.js";
import {
  rispostaErroreArgomentiTroppoGrandi,
  rispostaErroreValidazione,
} from "./lib/mcp-errors.js";
import { compila, estraiSegnaposti } from "./lib/template.js";
import { avvolgiContenutoNonFidato } from "./lib/untrusted-framing.js";

// ─── Vault path discovery ───

const IDENTIFIER = "com.pap.client";
const VAULT_FILENAME = "pap-vault.db";

function defaultVaultPath(): string {
  const home = homedir();
  const plat = platform();
  if (plat === "darwin") {
    return join(home, "Library", "Application Support", IDENTIFIER, VAULT_FILENAME);
  }
  if (plat === "win32") {
    const appData = process.env.APPDATA ?? join(home, "AppData", "Roaming");
    return join(appData, IDENTIFIER, VAULT_FILENAME);
  }
  // Linux + altri Unix
  const xdg = process.env.XDG_DATA_HOME ?? join(home, ".local", "share");
  return join(xdg, IDENTIFIER, VAULT_FILENAME);
}

const VAULT_PATH = process.env.PAP_VAULT_PATH ?? defaultVaultPath();

if (!existsSync(VAULT_PATH)) {
  process.stderr.write(
    `[pap-mcp-server] Vault non trovato: ${VAULT_PATH}\n` +
      `Imposta PAP_VAULT_PATH oppure crea il vault dal client desktop prima di avviare il server MCP.\n`,
  );
  process.exit(1);
}

// ─── Vault reader ───

interface PromptRow {
  id: string;
  title: string;
  description: string | null;
  body: string;
  visibility: string;
  target_model: string | null;
  is_favorite: number;
  use_count: number;
  last_used_at: string | null;
  version: number;
  created_at: string;
  updated_at: string;
}

interface PromptDettaglio extends PromptRow {
  tags: string[];
}

const db = new Database(VAULT_PATH, { readonly: true, fileMustExist: true });

function tagsPerPrompt(promptId: string): string[] {
  const stmt = db.prepare(
    `SELECT t.Name FROM Tags t
     JOIN PromptTags pt ON pt.TagId = t.Id
     WHERE pt.PromptId = ? AND t.DeletedAt IS NULL
     ORDER BY t.Name ASC`,
  );
  const rows = stmt.all(promptId) as { Name: string }[];
  return rows.map((r) => r.Name);
}

function promptCerca(
  query: string,
  limit: number,
  targetModel: string | null,
  tagsFilter: string[],
): PromptRow[] {
  const lim = clampLimit(limit);

  let sql: string;
  const params: unknown[] = [];

  if (!query.trim()) {
    sql = `SELECT Id as id, Title as title, Description as description, Body as body,
                  Visibility as visibility, TargetModel as target_model,
                  IsFavorite as is_favorite, UseCount as use_count,
                  LastUsedAt as last_used_at, Version as version,
                  CreatedAt as created_at, UpdatedAt as updated_at
           FROM Prompts AS p
           WHERE DeletedAt IS NULL`;
  } else {
    const fts = sanitizzaFts(query);
    if (!fts) return [];
    sql = `SELECT p.Id as id, p.Title as title, p.Description as description, p.Body as body,
                  p.Visibility as visibility, p.TargetModel as target_model,
                  p.IsFavorite as is_favorite, p.UseCount as use_count,
                  p.LastUsedAt as last_used_at, p.Version as version,
                  p.CreatedAt as created_at, p.UpdatedAt as updated_at
           FROM PromptsFts f
           JOIN Prompts p ON f.PromptId = p.Id
           WHERE PromptsFts MATCH ? AND p.DeletedAt IS NULL`;
    params.push(fts);
  }

  if (targetModel) {
    sql += ` AND p.TargetModel = ?`;
    params.push(targetModel);
  }

  sql += ` ORDER BY ${query.trim() ? "rank" : "COALESCE(p.LastUsedAt, p.UpdatedAt) DESC"} LIMIT ?`;
  params.push(lim);

  const rows = db.prepare(sql).all(...params) as PromptRow[];

  if (tagsFilter.length === 0) return rows;

  return rows.filter((p) => {
    const promptTags = tagsPerPrompt(p.id);
    return tagsFilter.every((t) => promptTags.includes(t));
  });
}

function promptGet(id: string): PromptDettaglio | null {
  const stmt = db.prepare(
    `SELECT Id as id, Title as title, Description as description, Body as body,
            Visibility as visibility, TargetModel as target_model,
            IsFavorite as is_favorite, UseCount as use_count,
            LastUsedAt as last_used_at, Version as version,
            CreatedAt as created_at, UpdatedAt as updated_at
     FROM Prompts WHERE Id = ? AND DeletedAt IS NULL`,
  );
  const row = stmt.get(id) as PromptRow | undefined;
  if (!row) return null;
  return { ...row, tags: tagsPerPrompt(id) };
}

function promptListRecent(limit: number): PromptRow[] {
  const lim = clampLimit(limit);
  return db
    .prepare(
      `SELECT Id as id, Title as title, Description as description, Body as body,
              Visibility as visibility, TargetModel as target_model,
              IsFavorite as is_favorite, UseCount as use_count,
              LastUsedAt as last_used_at, Version as version,
              CreatedAt as created_at, UpdatedAt as updated_at
       FROM Prompts
       WHERE DeletedAt IS NULL
       ORDER BY COALESCE(LastUsedAt, UpdatedAt) DESC
       LIMIT ?`,
    )
    .all(lim) as PromptRow[];
}

// ─── MCP Server setup ───

const server = new Server(
  {
    name: "pap-mcp-server",
    version: "0.1.0",
  },
  {
    capabilities: {
      tools: {},
    },
  },
);

const TOOLS = [
  {
    name: "pap_search",
    description:
      "Cerca prompt nel vault Prompt a Porter via FTS5. Filtra per modello target o tag se forniti.",
    inputSchema: {
      type: "object",
      properties: {
        query: { type: "string", description: "Testo di ricerca (vuoto = recenti)" },
        limit: { type: "number", description: "Max risultati (default 10, max 50)" },
        target_model: {
          type: "string",
          description: "Filtra per modello target (es. claude-sonnet)",
        },
        tags: {
          type: "array",
          items: { type: "string" },
          description: "Filtra: il prompt deve avere TUTTI questi tag",
        },
      },
    },
  },
  {
    name: "pap_get",
    description: "Restituisce il dettaglio completo di un prompt per ID, inclusi i tag.",
    inputSchema: {
      type: "object",
      properties: {
        prompt_id: { type: "string" },
      },
      required: ["prompt_id"],
    },
  },
  {
    name: "pap_list_recent",
    description: "Lista i prompt usati di recente, ordinati dal più recente.",
    inputSchema: {
      type: "object",
      properties: {
        limit: { type: "number", description: "Default 10, max 50" },
      },
    },
  },
  {
    name: "pap_render",
    description:
      "Compila un prompt con i valori forniti per i suoi segnaposti `{{nome}}`. Restituisce il testo finale.",
    inputSchema: {
      type: "object",
      properties: {
        prompt_id: { type: "string" },
        vars: {
          type: "object",
          description: "Mapping nome_segnaposto -> valore",
          additionalProperties: { type: "string" },
        },
      },
      required: ["prompt_id"],
    },
  },
];

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: TOOLS,
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  // Guardia aggregata economica PRIMA di qualunque safeParse per-tool:
  // rifiuta subito payload enormi (es. milioni di chiavi in `vars` o
  // elementi in `tags`) senza far attraversare l'intera struttura al
  // parser Zod, che altrimenti la validerebbe per intero prima di
  // scoprire che supera i limiti dichiarati nello schema.
  if (argsTroppoGrandi(args)) {
    return rispostaErroreArgomentiTroppoGrandi(name, MAX_ARGS_JSON_LENGTH);
  }

  try {
    switch (name) {
      case "pap_search": {
        const parsed = papSearchArgsSchema.safeParse(args ?? {});
        if (!parsed.success) {
          return rispostaErroreValidazione(name, parsed.error);
        }
        const a = parsed.data;
        const risultati = promptCerca(
          a.query ?? "",
          a.limit ?? 10,
          a.target_model ?? null,
          a.tags ?? [],
        );
        return {
          content: [
            {
              type: "text",
              // #462: title/description provengono dal vault utente →
              // avvolti in <untrusted_vault_content> per difesa da
              // prompt-injection indiretta.
              text: avvolgiContenutoNonFidato(
                JSON.stringify(
                  risultati.map((p) => ({
                    id: p.id,
                    title: p.title,
                    description: p.description,
                    visibility: p.visibility,
                    target_model: p.target_model,
                    use_count: p.use_count,
                    last_used_at: p.last_used_at,
                    version: p.version,
                  })),
                  null,
                  2,
                ),
              ),
            },
          ],
        };
      }

      case "pap_get": {
        const parsed = papGetArgsSchema.safeParse(args ?? {});
        if (!parsed.success) {
          return rispostaErroreValidazione(name, parsed.error);
        }
        const a = parsed.data;
        const p = promptGet(a.prompt_id);
        if (!p) {
          return {
            content: [{ type: "text", text: `Prompt ${a.prompt_id} non trovato` }],
            isError: true,
          };
        }
        return {
          content: [
            {
              type: "text",
              // #462: include title/description/body dal vault →
              // avvolti in <untrusted_vault_content> per difesa da
              // prompt-injection indiretta (il body è il payload più
              // sensibile, spesso incollato direttamente in un prompt).
              text: avvolgiContenutoNonFidato(
                JSON.stringify(
                  {
                    ...p,
                    is_favorite: p.is_favorite !== 0,
                    segnaposti: estraiSegnaposti(p.body),
                  },
                  null,
                  2,
                ),
              ),
            },
          ],
        };
      }

      case "pap_list_recent": {
        const parsed = papListRecentArgsSchema.safeParse(args ?? {});
        if (!parsed.success) {
          return rispostaErroreValidazione(name, parsed.error);
        }
        const a = parsed.data;
        const risultati = promptListRecent(a.limit ?? 10);
        return {
          content: [
            {
              type: "text",
              // #462: title proviene dal vault utente → avvolto in
              // <untrusted_vault_content> per difesa da prompt-injection
              // indiretta.
              text: avvolgiContenutoNonFidato(
                JSON.stringify(
                  risultati.map((p) => ({
                    id: p.id,
                    title: p.title,
                    last_used_at: p.last_used_at,
                    use_count: p.use_count,
                  })),
                  null,
                  2,
                ),
              ),
            },
          ],
        };
      }

      case "pap_render": {
        const parsed = papRenderArgsSchema.safeParse(args ?? {});
        if (!parsed.success) {
          return rispostaErroreValidazione(name, parsed.error);
        }
        const a = parsed.data;
        const p = promptGet(a.prompt_id);
        if (!p) {
          return {
            content: [{ type: "text", text: `Prompt ${a.prompt_id} non trovato` }],
            isError: true,
          };
        }
        const compilato = compila(p.body, a.vars ?? {});
        const segnapostiOriginali = estraiSegnaposti(p.body);
        const valori = a.vars ?? {};
        const segnapostiNonCompilati = segnapostiOriginali.filter(
          (s) => !valori[s]?.trim(),
        );
        return {
          content: [
            {
              type: "text",
              // #462: `compilato` è il body del prompt (dal vault utente),
              // compilato con i segnaposti — è il payload più esposto a
              // prompt-injection indiretta perché tipicamente viene
              // incollato direttamente in un altro prompt. La nota sui
              // segnaposti mancanti è metadato del server, resta fuori
              // dal tag di contenuto non fidato.
              text:
                avvolgiContenutoNonFidato(compilato) +
                (segnapostiNonCompilati.length > 0
                  ? `\n\n[NOTA: segnaposti non compilati: ${segnapostiNonCompilati.join(", ")}]`
                  : ""),
            },
          ],
        };
      }

      default:
        return {
          content: [{ type: "text", text: `Tool sconosciuto: ${name}` }],
          isError: true,
        };
    }
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    return {
      content: [{ type: "text", text: `Errore: ${msg}` }],
      isError: true,
    };
  }
});

// ─── Entry point ───

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  process.stderr.write(`[pap-mcp-server] Pronto. Vault: ${VAULT_PATH}\n`);
}

main().catch((err) => {
  process.stderr.write(`[pap-mcp-server] Fatale: ${err}\n`);
  process.exit(1);
});
