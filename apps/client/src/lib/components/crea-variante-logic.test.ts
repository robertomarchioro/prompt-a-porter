/**
 * Test per il flow "crea variante" (issue #559).
 *
 * La logica pura è estratta in crea-variante-logic.ts e importata qui per
 * evitare drift tra test e implementazione reale (_pure/_impl pattern, già
 * usato per nuova-cartella-logic.ts).
 *
 * Copre:
 * - payload invoke (parentId + etichetta trimmata/null)
 * - successo: dispatch pap:lista-mutata + pap:apri-prompt (in quest'ordine),
 *   con l'id restituito dal backend
 * - errore: nessun dispatch, messaggio propagato senza prefisso "Error: "
 */

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { eseguiCreaVariante } from "./crea-variante-logic";

type InvokeVarianteFn = (
  cmd: string,
  args: { parentId: string; etichetta: string | null },
) => Promise<string>;
type DispatchListaMutataFn = () => void;
type DispatchApriPromptFn = (id: string) => void;

describe("eseguiCreaVariante: payload invoke", () => {
  let invokeFn: Mock<InvokeVarianteFn>;
  let dispatchListaMutata: Mock<DispatchListaMutataFn>;
  let dispatchApriPrompt: Mock<DispatchApriPromptFn>;

  beforeEach(() => {
    invokeFn = vi.fn<InvokeVarianteFn>().mockResolvedValue("variant-id-123");
    dispatchListaMutata = vi.fn<DispatchListaMutataFn>();
    dispatchApriPrompt = vi.fn<DispatchApriPromptFn>();
  });

  it("chiama prompt_crea_variante con { parentId, etichetta: null } quando l'etichetta è vuota", async () => {
    await eseguiCreaVariante(
      "prompt-1",
      "",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(invokeFn).toHaveBeenCalledOnce();
    expect(invokeFn).toHaveBeenCalledWith("prompt_crea_variante", {
      parentId: "prompt-1",
      etichetta: null,
    });
  });

  it("trimma l'etichetta prima di invocare", async () => {
    await eseguiCreaVariante(
      "prompt-1",
      "  mobile  ",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(invokeFn).toHaveBeenCalledWith("prompt_crea_variante", {
      parentId: "prompt-1",
      etichetta: "mobile",
    });
  });

  it("passa null se l'etichetta è composta solo da spazi", async () => {
    await eseguiCreaVariante(
      "prompt-1",
      "   ",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(invokeFn).toHaveBeenCalledWith("prompt_crea_variante", {
      parentId: "prompt-1",
      etichetta: null,
    });
  });
});

describe("eseguiCreaVariante: side-effects al successo", () => {
  let invokeFn: Mock<InvokeVarianteFn>;
  let dispatchListaMutata: Mock<DispatchListaMutataFn>;
  let dispatchApriPrompt: Mock<DispatchApriPromptFn>;

  beforeEach(() => {
    invokeFn = vi.fn<InvokeVarianteFn>().mockResolvedValue("variant-id-abc");
    dispatchListaMutata = vi.fn<DispatchListaMutataFn>();
    dispatchApriPrompt = vi.fn<DispatchApriPromptFn>();
  });

  it("emette pap:lista-mutata esattamente una volta al successo (#559: la lista deve rinfrescarsi)", async () => {
    await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(dispatchListaMutata).toHaveBeenCalledOnce();
  });

  it("apre la nuova variante con l'id restituito dal backend", async () => {
    await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(dispatchApriPrompt).toHaveBeenCalledOnce();
    expect(dispatchApriPrompt).toHaveBeenCalledWith("variant-id-abc");
  });

  it("emette pap:lista-mutata prima di aprire la variante (ordine corretto)", async () => {
    const callOrder: string[] = [];
    dispatchListaMutata = vi
      .fn<DispatchListaMutataFn>()
      .mockImplementation(() => {
        callOrder.push("lista-mutata");
      });
    dispatchApriPrompt = vi
      .fn<DispatchApriPromptFn>()
      .mockImplementation(() => {
        callOrder.push("apri-prompt");
      });

    await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(callOrder).toEqual(["lista-mutata", "apri-prompt"]);
  });

  it("ritorna ok=true e l'id sul successo", async () => {
    const risultato = await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(risultato.ok).toBe(true);
    expect(risultato.id).toBe("variant-id-abc");
  });
});

describe("eseguiCreaVariante: errore backend — nessun dispatch", () => {
  let dispatchListaMutata: Mock<DispatchListaMutataFn>;
  let dispatchApriPrompt: Mock<DispatchApriPromptFn>;

  beforeEach(() => {
    dispatchListaMutata = vi.fn<DispatchListaMutataFn>();
    dispatchApriPrompt = vi.fn<DispatchApriPromptFn>();
  });

  it("non emette pap:lista-mutata se il backend fallisce", async () => {
    const invokeFn = vi
      .fn<InvokeVarianteFn>()
      .mockRejectedValue(new Error("DB error"));

    await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(dispatchListaMutata).not.toHaveBeenCalled();
  });

  it("non apre nessuna variante se il backend fallisce", async () => {
    const invokeFn = vi
      .fn<InvokeVarianteFn>()
      .mockRejectedValue(new Error("DB error"));

    await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(dispatchApriPrompt).not.toHaveBeenCalled();
  });

  it("propaga errore stringa dal backend rimuovendo prefisso 'Error: '", async () => {
    const invokeFn = vi
      .fn<InvokeVarianteFn>()
      .mockRejectedValue("Error: etichetta già in uso");

    const risultato = await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("etichetta già in uso");
  });

  it("propaga messaggio da istanza Error", async () => {
    const invokeFn = vi
      .fn<InvokeVarianteFn>()
      .mockRejectedValue(new Error("prompt non trovato"));

    const risultato = await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("prompt non trovato");
  });

  it("usa messaggio generico per errori non-stringa e non-Error", async () => {
    const invokeFn = vi
      .fn<InvokeVarianteFn>()
      .mockRejectedValue({ codice: 42 });

    const risultato = await eseguiCreaVariante(
      "prompt-1",
      "b",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
    );

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("Impossibile creare la variante.");
  });
});
