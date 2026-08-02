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

// Review PR #592 (LOW ma centrale): `@tauri-apps/plugin-log` chiama
// `invoke()` di `@tauri-apps/api/core`, che fuori da un contesto Tauri
// rifiuterebbe la Promise — mockato qui per poter leggere le righe scritte
// da `logInfoApp` (nRigheBackend/nRigheFiltrate) invece di limitarsi a
// osservare il fallback silenzioso su `console.*`.
const infoLogSpy = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
vi.mock("@tauri-apps/plugin-log", () => ({
  info: (msg: string) => infoLogSpy(msg),
  error: () => Promise.resolve(),
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
    infoLogSpy.mockClear();
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

  // Review PR #592 (LOW ma centrale): `nRigheFiltrate` deve riflettere i
  // dati APPENA caricati, non quelli della ricarica precedente — è il
  // discriminante chiave per #558. Due ricariche consecutive con insiemi
  // di dimensione diversa (e un filtro livello attivo, cosicché
  // nRigheFiltrate != nRigheBackend) dimostrano che il conteggio loggato
  // segue sempre i dati correnti.
  it("logga nRigheFiltrate calcolato sui dati appena caricati, non su quelli della ricarica precedente", async () => {
    // Arrange: primo caricamento, 3 righe (1 WARN).
    invokeMock.mockResolvedValueOnce(RIGHE_CRONOLOGICHE);
    const { container, getByLabelText } = render(LogViewer);
    await vi.waitFor(() => {
      expect(container.querySelectorAll(".log-riga")).toHaveLength(3);
    });

    // Filtra per WARN (nessun nuovo invoke): righeFiltrate passa a 1.
    const selectLivello = getByLabelText("Filtro livello") as HTMLSelectElement;
    await fireEvent.change(selectLivello, { target: { value: "WARN" } });
    await vi.waitFor(() => {
      expect(container.querySelectorAll(".log-riga")).toHaveLength(1);
    });
    infoLogSpy.mockClear();

    // Act: secondo caricamento, 5 righe di dimensione diversa dal primo,
    // di cui 3 WARN (nRigheFiltrate atteso = 3, non 1 = valore stantio
    // della ricarica precedente).
    const RIGHE_SECONDO_GIRO = [
      {
        timestamp: "2026-08-02 11:00:00",
        level: "WARN",
        target: "pap_lib::d",
        message: "warn-1",
        raw: "[2026-08-02][11:00:00][pap_lib::d][WARN] warn-1",
      },
      {
        timestamp: "2026-08-02 11:00:01",
        level: "WARN",
        target: "pap_lib::e",
        message: "warn-2",
        raw: "[2026-08-02][11:00:01][pap_lib::e][WARN] warn-2",
      },
      {
        timestamp: "2026-08-02 11:00:02",
        level: "WARN",
        target: "pap_lib::f",
        message: "warn-3",
        raw: "[2026-08-02][11:00:02][pap_lib::f][WARN] warn-3",
      },
      {
        timestamp: "2026-08-02 11:00:03",
        level: "ERROR",
        target: "pap_lib::g",
        message: "err-1",
        raw: "[2026-08-02][11:00:03][pap_lib::g][ERROR] err-1",
      },
      {
        timestamp: "2026-08-02 11:00:04",
        level: "ERROR",
        target: "pap_lib::h",
        message: "err-2",
        raw: "[2026-08-02][11:00:04][pap_lib::h][ERROR] err-2",
      },
    ];
    invokeMock.mockResolvedValueOnce(RIGHE_SECONDO_GIRO);
    const bottoneAggiorna = getByLabelText("Aggiorna ora") as HTMLButtonElement;
    await fireEvent.click(bottoneAggiorna);

    // Assert: la lista renderizzata riflette già il nuovo filtro/dataset...
    await vi.waitFor(() => {
      expect(container.querySelectorAll(".log-riga")).toHaveLength(3);
    });
    // ...e la riga di log dell'esito riporta lo stesso numero, non quello
    // (stantio) della ricarica precedente.
    await vi.waitFor(() => {
      const righeEsito = infoLogSpy.mock.calls
        .map((c) => c[0] as string)
        .filter((msg) => msg.includes("ricarica esito origine=manuale"));
      expect(righeEsito).toHaveLength(1);
      expect(righeEsito[0]).toContain("nRigheBackend=5");
      expect(righeEsito[0]).toContain("nRigheFiltrate=3");
    });
  });
});
