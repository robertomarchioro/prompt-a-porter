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

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { nomeValido, eseguiCreaCartella } from "./nuova-cartella-logic";

// Firma del mock invoke attesa da eseguiCreaCartella: tipizzata esplicitamente
// perché in vitest 4 `vi.fn()` ritorna Mock<Procedure | Constructable>, non più
// assegnabile a una firma specifica come argomento.
type InvokeFolderFn = (
  cmd: string,
  args: { dati: { nome: string; parent_folder_id: null } },
) => Promise<string>;
type VoidFn = () => void;

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
  let invokeFn: Mock<InvokeFolderFn>;
  let dispatchFn: Mock<VoidFn>;
  let onChiudi: Mock<VoidFn>;

  beforeEach(() => {
    invokeFn = vi.fn<InvokeFolderFn>().mockResolvedValue("folder-id-123");
    dispatchFn = vi.fn<VoidFn>();
    onChiudi = vi.fn<VoidFn>();
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
  let invokeFn: Mock<InvokeFolderFn>;
  let dispatchFn: Mock<VoidFn>;
  let onChiudi: Mock<VoidFn>;

  beforeEach(() => {
    invokeFn = vi.fn<InvokeFolderFn>().mockResolvedValue("folder-id-abc");
    dispatchFn = vi.fn<VoidFn>();
    onChiudi = vi.fn<VoidFn>();
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
    dispatchFn = vi.fn<VoidFn>().mockImplementation(() => { callOrder.push("dispatch"); });
    onChiudi = vi.fn<VoidFn>().mockImplementation(() => { callOrder.push("chiudi"); });

    await eseguiCreaCartella("NuovaCartella", invokeFn, dispatchFn, onChiudi);

    expect(callOrder).toEqual(["dispatch", "chiudi"]);
  });
});

// ── Submit: errori — dispatch e onChiudi NON chiamati ────────────────────────

describe("eseguiCreaCartella: errore backend — nessun side-effect", () => {
  let dispatchFn: Mock<VoidFn>;
  let onChiudi: Mock<VoidFn>;

  beforeEach(() => {
    dispatchFn = vi.fn<VoidFn>();
    onChiudi = vi.fn<VoidFn>();
  });

  it("non chiama dispatchFn se il backend fallisce", async () => {
    const invokeFn = vi.fn<InvokeFolderFn>().mockRejectedValue(new Error("DB error"));

    await eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi);

    expect(dispatchFn).not.toHaveBeenCalled();
  });

  it("non chiama onChiudi se il backend fallisce", async () => {
    const invokeFn = vi.fn<InvokeFolderFn>().mockRejectedValue(new Error("DB error"));

    await eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi);

    expect(onChiudi).not.toHaveBeenCalled();
  });

  it("non chiama dispatchFn né onChiudi per nome vuoto (validazione client)", async () => {
    const invokeFn = vi.fn<InvokeFolderFn>();

    await eseguiCreaCartella("", invokeFn, dispatchFn, onChiudi);

    expect(dispatchFn).not.toHaveBeenCalled();
    expect(onChiudi).not.toHaveBeenCalled();
  });

  it("non chiama dispatchFn né onChiudi per nome con slash", async () => {
    const invokeFn = vi.fn<InvokeFolderFn>();

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
      .fn<InvokeFolderFn>()
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
    const invokeFn = vi.fn<InvokeFolderFn>().mockRejectedValue(new Error("nome non valido"));

    const risultato = await eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("nome non valido");
  });

  it("usa messaggio generico per errori non-stringa e non-Error", async () => {
    const invokeFn = vi.fn<InvokeFolderFn>().mockRejectedValue({ codice: 42 });

    const risultato = await eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("Impossibile creare la cartella.");
  });

  it("ritorna ok=false senza lanciare eccezioni non gestite", async () => {
    const invokeFn = vi.fn<InvokeFolderFn>().mockRejectedValue("Error: server down");

    await expect(
      eseguiCreaCartella("Test", invokeFn, dispatchFn, onChiudi),
    ).resolves.toMatchObject({ ok: false });
  });

  it("ritorna ok=false per nome vuoto con messaggio descrittivo", async () => {
    const invokeFn = vi.fn<InvokeFolderFn>();
    const risultato = await eseguiCreaCartella("", invokeFn, dispatchFn, onChiudi);

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBeTruthy();
  });
});
