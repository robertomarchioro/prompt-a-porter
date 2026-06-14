/**
 * Test per il flow "nuova cartella" (issue #301).
 *
 * La logica pura è estratta in nuova-cartella-logic.ts e importata qui
 * per evitare drift tra test e implementazione reale (_pure/_impl pattern).
 *
 * Copre:
 * - nomeValido: stringa vuota, soli-spazi, "/" rifiutati; nomi validi
 * - eseguiCreaCartella: payload invoke, trim, successo con dispatch+onChiudi,
 *   errori (stringa/Error/unknown), guardia nome vuoto/slash
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { nomeValido, eseguiCreaCartella } from "./nuova-cartella-logic";

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

  it("ritorna false per nome con slash (specchia backend nome_valido)", () => {
    expect(nomeValido("foo/bar")).toBe(false);
  });

  it("ritorna false per nome che è solo uno slash", () => {
    expect(nomeValido("/")).toBe(false);
  });

  it("ritorna false per nome con slash dopo trim", () => {
    expect(nomeValido("  /  ")).toBe(false);
  });
});

// ── Submit: invoke folder_crea con payload corretto ──────────────────────────

describe("eseguiCreaCartella: payload invoke", () => {
  let invokeFn: ReturnType<typeof vi.fn>;
  let dispatchFn: ReturnType<typeof vi.fn>;
  let onChiudi: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    invokeFn = vi.fn().mockResolvedValue("folder-id-123");
    dispatchFn = vi.fn();
    onChiudi = vi.fn();
  });

  it("chiama folder_crea con { nome, parent_folder_id: null }", async () => {
    await eseguiCreaCartella("Marketing", invokeFn, dispatchFn, onChiudi);

    expect(invokeFn).toHaveBeenCalledOnce();
    expect(invokeFn).toHaveBeenCalledWith("folder_crea", {
      dati: { nome: "Marketing", parent_folder_id: null },
    });
  });

  it("trimma gli spazi dal nome prima di invocare", async () => {
    await eseguiCreaCartella("  Marketing  ", invokeFn, dispatchFn, onChiudi);

    expect(invokeFn).toHaveBeenCalledWith("folder_crea", {
      dati: { nome: "Marketing", parent_folder_id: null },
    });
  });

  it("ritorna ok=true e l'id sul successo", async () => {
    const risultato = await eseguiCreaCartella("Marketing", invokeFn, dispatchFn, onChiudi);

    expect(risultato.ok).toBe(true);
    expect(risultato.id).toBe("folder-id-123");
  });

  it("non chiama invoke per nome vuoto", async () => {
    await eseguiCreaCartella("", invokeFn, dispatchFn, onChiudi);
    expect(invokeFn).not.toHaveBeenCalled();
  });

  it("non chiama invoke per nome composto solo da spazi", async () => {
    await eseguiCreaCartella("   ", invokeFn, dispatchFn, onChiudi);
    expect(invokeFn).not.toHaveBeenCalled();
  });

  it("non chiama invoke per nome con slash", async () => {
    await eseguiCreaCartella("foo/bar", invokeFn, dispatchFn, onChiudi);
    expect(invokeFn).not.toHaveBeenCalled();
  });
});

// ── Submit: side-effects al successo ─────────────────────────────────────────

describe("eseguiCreaCartella: side-effects al successo", () => {
  let invokeFn: ReturnType<typeof vi.fn>;
  let dispatchFn: ReturnType<typeof vi.fn>;
  let onChiudi: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    invokeFn = vi.fn().mockResolvedValue("folder-id-abc");
    dispatchFn = vi.fn();
    onChiudi = vi.fn();
  });

  it("chiama dispatchFn (pap:lista-mutata) esattamente una volta al successo", async () => {
    await eseguiCreaCartella("NuovaCartella", invokeFn, dispatchFn, onChiudi);

    expect(dispatchFn).toHaveBeenCalledOnce();
  });

  it("chiama onChiudi esattamente una volta al successo", async () => {
    await eseguiCreaCartella("NuovaCartella", invokeFn, dispatchFn, onChiudi);

    expect(onChiudi).toHaveBeenCalledOnce();
  });

  it("chiama dispatchFn prima di onChiudi (ordine corretto)", async () => {
    const callOrder: string[] = [];
    dispatchFn = vi.fn().mockImplementation(() => { callOrder.push("dispatch"); });
    onChiudi = vi.fn().mockImplementation(() => { callOrder.push("chiudi"); });

    await eseguiCreaCartella("NuovaCartella", invokeFn, dispatchFn, onChiudi);

    expect(callOrder).toEqual(["dispatch", "chiudi"]);
  });
});

// ── Submit: errori — dispatch e onChiudi NON chiamati ────────────────────────

describe("eseguiCreaCartella: errore backend — nessun side-effect", () => {
  let dispatchFn: ReturnType<typeof vi.fn>;
  let onChiudi: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    dispatchFn = vi.fn();
    onChiudi = vi.fn();
  });

  it("non chiama dispatchFn se il backend fallisce", async () => {
    const invokeFn = vi.fn().mockRejectedValue(new Error("DB error"));

    await eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi);

    expect(dispatchFn).not.toHaveBeenCalled();
  });

  it("non chiama onChiudi se il backend fallisce", async () => {
    const invokeFn = vi.fn().mockRejectedValue(new Error("DB error"));

    await eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi);

    expect(onChiudi).not.toHaveBeenCalled();
  });

  it("non chiama dispatchFn né onChiudi per nome vuoto (validazione client)", async () => {
    const invokeFn = vi.fn();

    await eseguiCreaCartella("", invokeFn, dispatchFn, onChiudi);

    expect(dispatchFn).not.toHaveBeenCalled();
    expect(onChiudi).not.toHaveBeenCalled();
  });

  it("non chiama dispatchFn né onChiudi per nome con slash", async () => {
    const invokeFn = vi.fn();

    await eseguiCreaCartella("foo/bar", invokeFn, dispatchFn, onChiudi);

    expect(dispatchFn).not.toHaveBeenCalled();
    expect(onChiudi).not.toHaveBeenCalled();
  });
});

// ── Submit: errori non ingoiati silenziosamente ───────────────────────────────

describe("eseguiCreaCartella: gestione errori", () => {
  const dispatchFn = vi.fn();
  const onChiudi = vi.fn();

  it("propaga errore stringa dal backend rimuovendo prefisso 'Error: '", async () => {
    const invokeFn = vi
      .fn()
      .mockRejectedValue(
        "Error: Impossibile creare cartella: UNIQUE constraint failed",
      );

    const risultato = await eseguiCreaCartella("Doppio", invokeFn, dispatchFn, onChiudi);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe(
      "Impossibile creare cartella: UNIQUE constraint failed",
    );
  });

  it("propaga messaggio da istanza Error", async () => {
    const invokeFn = vi.fn().mockRejectedValue(new Error("nome non valido"));

    const risultato = await eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("nome non valido");
  });

  it("usa messaggio generico per errori non-stringa e non-Error", async () => {
    const invokeFn = vi.fn().mockRejectedValue({ codice: 42 });

    const risultato = await eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("Impossibile creare la cartella.");
  });

  it("ritorna ok=false senza lanciare eccezioni non gestite", async () => {
    const invokeFn = vi.fn().mockRejectedValue("Error: server down");

    await expect(
      eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi),
    ).resolves.toMatchObject({ ok: false });
  });

  it("ritorna ok=false per nome vuoto con messaggio descrittivo", async () => {
    const invokeFn = vi.fn();
    const risultato = await eseguiCreaCartella("", invokeFn, dispatchFn, onChiudi);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBeTruthy();
  });
});
