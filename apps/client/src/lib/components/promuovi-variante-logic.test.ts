/**
 * Test per il flow "promuovi variante a principale" (issue #572).
 *
 * La logica pura è estratta in promuovi-variante-logic.ts e importata qui
 * per evitare drift tra test e implementazione reale (_pure/_impl pattern,
 * già usato per crea-variante-logic.ts).
 *
 * Copre:
 * - payload invoke (variantId)
 * - successo: dispatch pap:lista-mutata + pap:apri-prompt + pap:prompt-promosso
 *   (in quest'ordine), quest'ultimo SEMPRE emesso anche quando l'id promosso
 *   coincide col prompt già aperto (causa radice #572: riassegnare un
 *   primitivo invariato è un no-op per la reattività di Svelte 5, quindi
 *   serve un evento dedicato che DetailPane confronta esplicitamente)
 * - errore: nessun dispatch, messaggio propagato senza prefisso "Error: "
 */

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { eseguiPromuoviVariante } from "./promuovi-variante-logic";

type InvokePromuoviFn = (
  cmd: string,
  args: { variantId: string },
) => Promise<void>;
type DispatchListaMutataFn = () => void;
type DispatchApriPromptFn = (id: string) => void;
type DispatchPromptPromossoFn = (id: string) => void;

describe("eseguiPromuoviVariante: payload invoke", () => {
  let invokeFn: Mock<InvokePromuoviFn>;
  let dispatchListaMutata: Mock<DispatchListaMutataFn>;
  let dispatchApriPrompt: Mock<DispatchApriPromptFn>;
  let dispatchPromptPromosso: Mock<DispatchPromptPromossoFn>;

  beforeEach(() => {
    invokeFn = vi.fn<InvokePromuoviFn>().mockResolvedValue(undefined);
    dispatchListaMutata = vi.fn<DispatchListaMutataFn>();
    dispatchApriPrompt = vi.fn<DispatchApriPromptFn>();
    dispatchPromptPromosso = vi.fn<DispatchPromptPromossoFn>();
  });

  it("chiama prompt_promuovi_variante con { variantId }", async () => {
    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(invokeFn).toHaveBeenCalledOnce();
    expect(invokeFn).toHaveBeenCalledWith("prompt_promuovi_variante", {
      variantId: "prm-variante-1",
    });
  });
});

describe("eseguiPromuoviVariante: side-effects al successo", () => {
  let invokeFn: Mock<InvokePromuoviFn>;
  let dispatchListaMutata: Mock<DispatchListaMutataFn>;
  let dispatchApriPrompt: Mock<DispatchApriPromptFn>;
  let dispatchPromptPromosso: Mock<DispatchPromptPromossoFn>;

  beforeEach(() => {
    invokeFn = vi.fn<InvokePromuoviFn>().mockResolvedValue(undefined);
    dispatchListaMutata = vi.fn<DispatchListaMutataFn>();
    dispatchApriPrompt = vi.fn<DispatchApriPromptFn>();
    dispatchPromptPromosso = vi.fn<DispatchPromptPromossoFn>();
  });

  it("emette pap:lista-mutata esattamente una volta al successo", async () => {
    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(dispatchListaMutata).toHaveBeenCalledOnce();
  });

  it("apre il prompt promosso con l'id passato", async () => {
    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(dispatchApriPrompt).toHaveBeenCalledOnce();
    expect(dispatchApriPrompt).toHaveBeenCalledWith("prm-variante-1");
  });

  it("#572: emette SEMPRE pap:prompt-promosso, anche a id invariato (caso 'Promuovi a principale' sulla variante già aperta)", async () => {
    // Riproduce esattamente lo scenario del bottone in RightRail:
    // variantId === id del prompt già aperto → pap:apri-prompt da solo
    // sarebbe un no-op reattivo. pap:prompt-promosso deve comunque partire
    // così DetailPane può forzare il refetch.
    await eseguiPromuoviVariante(
      "prm-aperto",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(dispatchPromptPromosso).toHaveBeenCalledOnce();
    expect(dispatchPromptPromosso).toHaveBeenCalledWith("prm-aperto");
  });

  it("emette gli eventi nell'ordine lista-mutata, apri-prompt, prompt-promosso", async () => {
    const callOrder: string[] = [];
    dispatchListaMutata = vi
      .fn<DispatchListaMutataFn>()
      .mockImplementation(() => callOrder.push("lista-mutata"));
    dispatchApriPrompt = vi
      .fn<DispatchApriPromptFn>()
      .mockImplementation(() => callOrder.push("apri-prompt"));
    dispatchPromptPromosso = vi
      .fn<DispatchPromptPromossoFn>()
      .mockImplementation(() => callOrder.push("prompt-promosso"));

    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(callOrder).toEqual(["lista-mutata", "apri-prompt", "prompt-promosso"]);
  });

  it("ritorna ok=true sul successo", async () => {
    const risultato = await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(risultato.ok).toBe(true);
  });
});

describe("eseguiPromuoviVariante: errore backend — nessun dispatch", () => {
  let dispatchListaMutata: Mock<DispatchListaMutataFn>;
  let dispatchApriPrompt: Mock<DispatchApriPromptFn>;
  let dispatchPromptPromosso: Mock<DispatchPromptPromossoFn>;

  beforeEach(() => {
    dispatchListaMutata = vi.fn<DispatchListaMutataFn>();
    dispatchApriPrompt = vi.fn<DispatchApriPromptFn>();
    dispatchPromptPromosso = vi.fn<DispatchPromptPromossoFn>();
  });

  it("non emette nessun evento se il backend fallisce", async () => {
    const invokeFn = vi
      .fn<InvokePromuoviFn>()
      .mockRejectedValue(new Error("DB error"));

    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(dispatchListaMutata).not.toHaveBeenCalled();
    expect(dispatchApriPrompt).not.toHaveBeenCalled();
    expect(dispatchPromptPromosso).not.toHaveBeenCalled();
  });

  it("propaga errore stringa dal backend rimuovendo prefisso 'Error: '", async () => {
    const invokeFn = vi
      .fn<InvokePromuoviFn>()
      .mockRejectedValue("Error: prompt non trovato");

    const risultato = await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("prompt non trovato");
  });

  it("propaga messaggio da istanza Error", async () => {
    const invokeFn = vi
      .fn<InvokePromuoviFn>()
      .mockRejectedValue(new Error("non e' una variante"));

    const risultato = await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("non e' una variante");
  });

  it("usa messaggio generico per errori non-stringa e non-Error", async () => {
    const invokeFn = vi
      .fn<InvokePromuoviFn>()
      .mockRejectedValue({ codice: 42 });

    const risultato = await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      dispatchListaMutata,
      dispatchApriPrompt,
      dispatchPromptPromosso,
    );

    expect(risultato.ok).toBe(false);
    expect(risultato.errore).toBe("Impossibile promuovere la variante.");
  });
});

describe("eseguiPromuoviVariante: strumentazione diagnostica (#572)", () => {
  it("senza il parametro log opzionale, non lancia e si comporta come prima", async () => {
    const invokeFn = vi.fn<InvokePromuoviFn>().mockResolvedValue(undefined);

    await expect(
      eseguiPromuoviVariante(
        "prm-variante-1",
        invokeFn,
        vi.fn<DispatchListaMutataFn>(),
        vi.fn<DispatchApriPromptFn>(),
        vi.fn<DispatchPromptPromossoFn>(),
      ),
    ).resolves.toEqual({ ok: true });
  });

  it("logga invoke e i tre dispatch, in ordine, al successo", async () => {
    const invokeFn = vi.fn<InvokePromuoviFn>().mockResolvedValue(undefined);
    const log = vi.fn<(messaggio: string) => void>();

    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      vi.fn<DispatchListaMutataFn>(),
      vi.fn<DispatchApriPromptFn>(),
      vi.fn<DispatchPromptPromossoFn>(),
      log,
    );

    const righe = log.mock.calls.map((c) => c[0]);
    expect(righe).toEqual([
      "[promuovi-variante] invoke prompt_promuovi_variante — avvio",
      "[promuovi-variante] invoke prompt_promuovi_variante — completato",
      "[promuovi-variante] dispatch pap:lista-mutata",
      "[promuovi-variante] dispatch pap:apri-prompt",
      "[promuovi-variante] dispatch pap:prompt-promosso",
    ]);
  });

  it("con un solo callback `log` (retrocompatibilità), l'errore ricade sullo stesso", async () => {
    const invokeFn = vi
      .fn<InvokePromuoviFn>()
      .mockRejectedValue(new Error("DB error"));
    const log = vi.fn<(messaggio: string) => void>();

    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      vi.fn<DispatchListaMutataFn>(),
      vi.fn<DispatchApriPromptFn>(),
      vi.fn<DispatchPromptPromossoFn>(),
      log,
    );

    const righe = log.mock.calls.map((c) => c[0]);
    expect(righe).toEqual([
      "[promuovi-variante] invoke prompt_promuovi_variante — avvio",
      "[promuovi-variante] invoke prompt_promuovi_variante — errore DB error",
    ]);
  });

  // Review PR #592 (MEDIUM): il 6° parametro passato da RightRail.svelte
  // era sempre `logInfoApp`, quindi la riga d'errore finiva a livello
  // info — invisibile col filtro di default `warn` del log applicativo.
  // Il 7° parametro opzionale `logErrore` separa i due livelli.
  it("con `logErrore` distinto da `log`, l'errore va SOLO a `logErrore` (mai a `log`)", async () => {
    const invokeFn = vi
      .fn<InvokePromuoviFn>()
      .mockRejectedValue(new Error("DB error"));
    const log = vi.fn<(messaggio: string) => void>();
    const logErrore = vi.fn<(messaggio: string) => void>();

    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      vi.fn<DispatchListaMutataFn>(),
      vi.fn<DispatchApriPromptFn>(),
      vi.fn<DispatchPromptPromossoFn>(),
      log,
      logErrore,
    );

    expect(log.mock.calls.map((c) => c[0])).toEqual([
      "[promuovi-variante] invoke prompt_promuovi_variante — avvio",
    ]);
    expect(logErrore.mock.calls.map((c) => c[0])).toEqual([
      "[promuovi-variante] invoke prompt_promuovi_variante — errore DB error",
    ]);
  });

  it("al successo `logErrore` distinto non viene mai chiamato", async () => {
    const invokeFn = vi.fn<InvokePromuoviFn>().mockResolvedValue(undefined);
    const logErrore = vi.fn<(messaggio: string) => void>();

    await eseguiPromuoviVariante(
      "prm-variante-1",
      invokeFn,
      vi.fn<DispatchListaMutataFn>(),
      vi.fn<DispatchApriPromptFn>(),
      vi.fn<DispatchPromptPromossoFn>(),
      vi.fn<(messaggio: string) => void>(),
      logErrore,
    );

    expect(logErrore).not.toHaveBeenCalled();
  });
});
