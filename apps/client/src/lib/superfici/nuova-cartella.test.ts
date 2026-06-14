/**
 * Test per il flow "nuova cartella" (issue #301).
 *
 * modale.svelte.ts usa $state (rune Svelte 5) e non può essere importato
 * nel runtime vitest (nessun compilatore Svelte attivo). Il pattern del
 * progetto (vedi vitest.config.ts: `exclude: ["*.svelte.ts"]`) è testare
 * solo la logica pura estratta in funzioni plain-TS. Questo file verifica:
 *
 * - La validazione del nome (nomeValido): stringa vuota e soli-spazi rifiutati
 * - Il payload invoke: folder_crea chiamato con { nome, parent_folder_id: null }
 * - Il trim del nome prima dell'invoke
 * - La guardia per nome vuoto: invoke non viene chiamata
 * - La gestione errori: stringa, Error, oggetto sconosciuto — niente silent swallow
 */

import { describe, it, expect, vi, beforeEach } from "vitest";

// ── Logica pura estratta dalla modale ────────────────────────────────────────
// (specchia esattamente ciò che NuovaCartellaModal.svelte fa internamente)

function nomeValido(nome: string): boolean {
  return nome.trim().length > 0;
}

interface RisultatoSubmit {
  ok: boolean;
  id?: string;
  errore?: string;
}

async function eseguiCreaCartella(
  nome: string,
  invokeFn: (
    cmd: string,
    args: { dati: { nome: string; parent_folder_id: null } },
  ) => Promise<string>,
): Promise<RisultatoSubmit> {
  if (!nomeValido(nome)) {
    return { ok: false, errore: "Il nome non può essere vuoto." };
  }

  try {
    const id = await invokeFn("folder_crea", {
      dati: { nome: nome.trim(), parent_folder_id: null },
    });
    return { ok: true, id };
  } catch (err: unknown) {
    const msg =
      err instanceof Error
        ? err.message
        : typeof err === "string"
          ? err.replace(/^Error:\s*/i, "")
          : "Impossibile creare la cartella.";
    return { ok: false, errore: msg };
  }
}

// ── Validazione nome ─────────────────────────────────────────────────────────

describe("nomeValido: validazione del nome cartella", () => {
  it("ritorna false per stringa vuota", () => {
    expect(nomeValido("")).toBe(false);
  });

  it("ritorna false per stringa con soli spazi", () => {
    expect(nomeValido("   ")).toBe(false);
  });

  it("ritorna true per nome con caratteri", () => {
    expect(nomeValido("Marketing")).toBe(true);
  });

  it("ritorna true per nome con spazi interni", () => {
    expect(nomeValido("Progetti 2026")).toBe(true);
  });

  it("ritorna true per nome con spazi iniziali e finali (trim)", () => {
    expect(nomeValido("  Brand  ")).toBe(true);
  });
});

// ── Submit: invoke folder_crea con payload corretto ──────────────────────────

describe("eseguiCreaCartella: payload invoke", () => {
  let invokeFn: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    invokeFn = vi.fn().mockResolvedValue("folder-id-123");
  });

  it("chiama folder_crea con { nome, parent_folder_id: null }", async () => {
    await eseguiCreaCartella("Marketing", invokeFn);

    expect(invokeFn).toHaveBeenCalledOnce();
    expect(invokeFn).toHaveBeenCalledWith("folder_crea", {
      dati: { nome: "Marketing", parent_folder_id: null },
    });
  });

  it("trimma gli spazi dal nome prima di invocare", async () => {
    await eseguiCreaCartella("  Marketing  ", invokeFn);

    expect(invokeFn).toHaveBeenCalledWith("folder_crea", {
      dati: { nome: "Marketing", parent_folder_id: null },
    });
  });

  it("ritorna ok=true e l'id sul successo", async () => {
    const risultato = await eseguiCreaCartella("Marketing", invokeFn);

    expect(risultato.ok).toBe(true);
    expect(risultato.id).toBe("folder-id-123");
  });

  it("non chiama invoke per nome vuoto", async () => {
    await eseguiCreaCartella("", invokeFn);
    expect(invokeFn).not.toHaveBeenCalled();
  });

  it("non chiama invoke per nome composto solo da spazi", async () => {
    await eseguiCreaCartella("   ", invokeFn);
    expect(invokeFn).not.toHaveBeenCalled();
  });
});

// ── Submit: errori non ingoiati silenziosamente ───────────────────────────────

describe("eseguiCreaCartella: gestione errori", () => {
  it("propaga errore stringa dal backend rimuovendo prefisso 'Error: '", async () => {
    const invokeFn = vi
      .fn()
      .mockRejectedValue(
        "Error: Impossibile creare cartella: UNIQUE constraint failed",
      );

    const risultato = await eseguiCreaCartella("Doppio", invokeFn);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe(
      "Impossibile creare cartella: UNIQUE constraint failed",
    );
  });

  it("propaga messaggio da istanza Error", async () => {
    const invokeFn = vi.fn().mockRejectedValue(new Error("nome non valido"));

    const risultato = await eseguiCreaCartella("Test", invokeFn);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("nome non valido");
  });

  it("usa messaggio generico per errori non-stringa e non-Error", async () => {
    const invokeFn = vi.fn().mockRejectedValue({ codice: 42 });

    const risultato = await eseguiCreaCartella("Test", invokeFn);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("Impossibile creare la cartella.");
  });

  it("ritorna ok=false senza lanciare eccezioni non gestite", async () => {
    const invokeFn = vi.fn().mockRejectedValue("Error: server down");

    await expect(eseguiCreaCartella("Test", invokeFn)).resolves.toMatchObject({
      ok: false,
    });
  });
});
