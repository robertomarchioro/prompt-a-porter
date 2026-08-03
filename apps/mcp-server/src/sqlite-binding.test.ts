import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import Database from "better-sqlite3";

/**
 * Smoke test del binding nativo di `better-sqlite3`.
 *
 * Perché esiste: fino alla v13 nessun test di questo pacchetto importava
 * `better-sqlite3`. I cinque file sotto `src/lib/` coprono logica pura, e la
 * CI (`mcp-server-build.yml`) esegue `tsc` + build + coverage — quindi
 * **compilava** il progetto senza mai **caricare** il modulo nativo. Un
 * binding rotto (prebuilt mancante per la piattaforma, versione N-API
 * incompatibile, glibc) sarebbe passato con la CI verde e sarebbe esploso
 * solo in mano all'utente, all'apertura del vault.
 *
 * La v13.0.0 è la prima versione riscritta su **N-API**, con i prebuilt
 * pubblicati direttamente nel pacchetto al posto di `prebuild-install`: è
 * esattamente il tipo di cambiamento che un type check non può validare.
 *
 * Questo test non verifica la logica dell'MCP server: verifica che il modulo
 * si carichi e che le quattro chiamate realmente usate da `src/index.ts`
 * funzionino — `new Database(...)` con le stesse opzioni, `prepare`, `all`,
 * `get`. Se un domani il binding smettesse di caricarsi, questo diventa
 * rosso invece di lasciar passare il problema.
 */

const tempDir = mkdtempSync(join(tmpdir(), "pap-sqlite-smoke-"));

afterAll(() => {
  rmSync(tempDir, { recursive: true, force: true });
});

describe("binding nativo better-sqlite3", () => {
  it("carica il modulo e apre un database in memoria", () => {
    const db = new Database(":memory:");
    expect(db.open).toBe(true);
    db.close();
  });

  it("esegue il giro di chiamate usato da index.ts: prepare, all, get", () => {
    const db = new Database(":memory:");
    db.exec(
      "CREATE TABLE Prompts (Id TEXT PRIMARY KEY, Title TEXT, Body TEXT)",
    );
    db.prepare("INSERT INTO Prompts (Id, Title, Body) VALUES (?, ?, ?)").run(
      "p1",
      "Primo",
      "corpo uno",
    );
    db.prepare("INSERT INTO Prompts (Id, Title, Body) VALUES (?, ?, ?)").run(
      "p2",
      "Secondo",
      "corpo due",
    );

    const righe = db.prepare("SELECT Id, Title FROM Prompts ORDER BY Id").all();
    expect(righe).toHaveLength(2);

    const riga = db.prepare("SELECT Title FROM Prompts WHERE Id = ?").get("p2");
    expect(riga).toEqual({ Title: "Secondo" });

    db.close();
  });

  it("rispetta fileMustExist: apre un file esistente e rifiuta uno inesistente", () => {
    // `index.ts` apre il vault con { readonly: true, fileMustExist: true }:
    // qui si verifica che entrambe le opzioni arrivino davvero al livello
    // nativo, non solo che TypeScript le accetti.
    const percorso = join(tempDir, "vault.db");
    const creato = new Database(percorso);
    creato.exec("CREATE TABLE Prompts (Id TEXT PRIMARY KEY)");
    creato.close();

    const solaLettura = new Database(percorso, {
      readonly: true,
      fileMustExist: true,
    });
    expect(solaLettura.readonly).toBe(true);
    expect(() =>
      solaLettura.prepare("INSERT INTO Prompts (Id) VALUES (?)").run("x"),
    ).toThrow();
    solaLettura.close();

    expect(
      () =>
        new Database(join(tempDir, "inesistente.db"), {
          readonly: true,
          fileMustExist: true,
        }),
    ).toThrow();
  });
});
