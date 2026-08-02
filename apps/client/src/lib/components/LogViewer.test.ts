// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/svelte";
import LogViewer from "./LogViewer.svelte";

// #558: il viewer log live va indagato end-to-end (non solo la logica pura
// in log-viewer-logic.test.ts) per verificare che il filtro "Tutti", il
// pulsante "Aggiorna" e il refetch all'apertura del pannello si comportino
// come descritto dal collaudo macOS.
const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

const RIGHE_CRONOLOGICHE = [
  {
    timestamp: "2026-08-02 10:00:00",
    level: "INFO",
    target: "pap_lib::a",
    message: "il più vecchio",
    raw: "[2026-08-02][10:00:00][pap_lib::a][INFO] il più vecchio",
  },
  {
    timestamp: "2026-08-02 10:00:01",
    level: "WARN",
    target: "pap_lib::b",
    message: "intermedio",
    raw: "[2026-08-02][10:00:01][pap_lib::b][WARN] intermedio",
  },
  {
    timestamp: "2026-08-02 10:00:02",
    level: "ERROR",
    target: "pap_lib::c",
    message: "il più recente",
    raw: "[2026-08-02][10:00:02][pap_lib::c][ERROR] il più recente",
  },
];

describe("LogViewer", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  afterEach(cleanup);

  it('con filtro "Tutti" mostra tutte le righe caricate, più recenti in cima', async () => {
    // Arrange
    invokeMock.mockResolvedValue(RIGHE_CRONOLOGICHE);

    // Act
    const { container } = render(LogViewer);
    await vi.waitFor(() => {
      expect(container.querySelectorAll(".log-riga")).toHaveLength(3);
    });

    // Assert: ordine invertito rispetto a quello cronologico del backend.
    const messaggi = Array.from(container.querySelectorAll(".log-msg")).map(
      (el) => el.textContent,
    );
    expect(messaggi).toEqual(["il più recente", "intermedio", "il più vecchio"]);
  });

  it("filtrando per un livello specifico mostra solo le righe corrispondenti (mai di più di Tutti)", async () => {
    // Arrange
    invokeMock.mockResolvedValue(RIGHE_CRONOLOGICHE);
    const { container, getByLabelText } = render(LogViewer);
    await vi.waitFor(() => {
      expect(container.querySelectorAll(".log-riga")).toHaveLength(3);
    });

    // Act
    const selectLivello = getByLabelText("Filtro livello") as HTMLSelectElement;
    await fireEvent.change(selectLivello, { target: { value: "WARN" } });

    // Assert
    const righe = container.querySelectorAll(".log-riga");
    expect(righe).toHaveLength(1);
    expect(righe[0].querySelector(".log-msg")?.textContent).toBe("intermedio");
  });

  it("il pulsante Aggiorna resta utilizzabile anche dopo un fallimento del caricamento", async () => {
    // Arrange: il primo invoke fallisce (rete/IPC), il secondo riesce.
    invokeMock
      .mockRejectedValueOnce(new Error("lettura log fallita"))
      .mockResolvedValueOnce(RIGHE_CRONOLOGICHE);

    const { container, getByLabelText } = render(LogViewer);

    // Assert: l'errore è mostrato (non ingoiato) e il pulsante non resta
    // bloccato — torna subito cliccabile grazie al finally.
    await vi.waitFor(() => {
      expect(container.querySelector(".log-err")?.textContent).toContain(
        "lettura log fallita",
      );
    });
    const bottoneAggiorna = getByLabelText("Aggiorna ora") as HTMLButtonElement;
    expect(bottoneAggiorna.disabled).toBe(false);

    // Act: un secondo click deve poter ricaricare con successo.
    await fireEvent.click(bottoneAggiorna);

    // Assert
    await vi.waitFor(() => {
      expect(container.querySelectorAll(".log-riga")).toHaveLength(3);
    });
    expect(container.querySelector(".log-err")).toBeNull();
  });

  it("ricarica quando il pannello passa da chiuso (aperto=false) ad aperto (aperto=true)", async () => {
    // Arrange
    invokeMock.mockResolvedValue(RIGHE_CRONOLOGICHE);
    const { container, rerender } = render(LogViewer, {
      props: { aperto: false },
    });

    // Il mount chiama comunque ricarica() una volta (onMount), a
    // prescindere da `aperto`. Attendiamo che quella prima chiamata sia
    // COMPLETATA (righe renderizzate, non solo invoke() invocato) prima di
    // aprire il pannello: `ricarica()` è rientrante (`if (inAttesa) return`)
    // quindi una seconda invocazione mentre la prima è ancora in corso
    // verrebbe scartata silenziosamente, e il test asserirebbe la cosa
    // sbagliata.
    await vi.waitFor(() => {
      expect(container.querySelectorAll(".log-riga")).toHaveLength(3);
    });
    expect(invokeMock).toHaveBeenCalledTimes(1);

    // Act
    await rerender({ aperto: true });

    // Assert: una seconda chiamata scatta esplicitamente all'apertura.
    await vi.waitFor(() => {
      expect(invokeMock).toHaveBeenCalledTimes(2);
    });
  });

  it("con il log vuoto mostra il messaggio di stato vuoto, non una lista bloccata", async () => {
    // Arrange
    invokeMock.mockResolvedValue([]);

    // Act
    const { container, getByLabelText } = render(LogViewer);
    const bottoneAggiorna = getByLabelText("Aggiorna ora") as HTMLButtonElement;

    // Assert: ".log-vuoto" compare già allo stato iniziale (righe=[] prima
    // ancora che ricarica() risolva), quindi va atteso il completamento
    // del caricamento (pulsante ri-abilitato) e non solo la comparsa del
    // messaggio, altrimenti si intercetta lo stato transitorio "in corso".
    await vi.waitFor(() => {
      expect(container.querySelector(".log-vuoto")).not.toBeNull();
      expect(bottoneAggiorna.disabled).toBe(false);
    });
  });
});
