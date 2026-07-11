import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { validaServerUrl, syncAvvia, syncFerma } from "./sync";

describe("validaServerUrl (fix #455)", () => {
  it("accetta un URL https:// valido", () => {
    expect(() => validaServerUrl("https://sync.example.com")).not.toThrow();
  });

  it("accetta un URL https:// con path/porta", () => {
    expect(() =>
      validaServerUrl("https://sync.example.com:8443/api"),
    ).not.toThrow();
  });

  it("rifiuta http:// verso host remoto", () => {
    expect(() => validaServerUrl("http://sync.example.com")).toThrow(
      /https:\/\//,
    );
  });

  it("rifiuta http:// anche verso localhost (nessun concetto di dev-mode nel client)", () => {
    expect(() => validaServerUrl("http://localhost:8080")).toThrow(
      /https:\/\//,
    );
  });

  it("rifiuta http:// verso 127.0.0.1", () => {
    expect(() => validaServerUrl("http://127.0.0.1:8080")).toThrow();
  });

  it("rifiuta schemi non-http del tutto estranei (es. ftp://, file://)", () => {
    expect(() => validaServerUrl("ftp://sync.example.com")).toThrow();
    expect(() => validaServerUrl("file:///etc/passwd")).toThrow();
  });

  it("rifiuta stringhe non analizzabili come URL", () => {
    expect(() => validaServerUrl("non-un-url")).toThrow();
    expect(() => validaServerUrl("")).toThrow();
  });

  it("il messaggio d'errore e' orientato all'utente (menziona https)", () => {
    try {
      validaServerUrl("http://esempio.it");
      expect.unreachable("doveva lanciare");
    } catch (e) {
      expect(e).toBeInstanceOf(Error);
      expect((e as Error).message).toContain("https://");
    }
  });
});

// Fix #455 (review HIGH-1): il token deve viaggiare SOLO come
// sub-protocollo WebSocket (Sec-WebSocket-Protocol), mai in query string.
describe("connettiWs (fix #455 HIGH-1 — token via Sec-WebSocket-Protocol)", () => {
  class MockWebSocket {
    static ultimaChiamata: { url: string; protocols?: string | string[] } | null =
      null;
    url: string;
    protocols?: string | string[];
    onopen: (() => void) | null = null;
    onmessage: ((e: MessageEvent) => void) | null = null;
    onclose: (() => void) | null = null;
    onerror: (() => void) | null = null;

    constructor(url: string, protocols?: string | string[]) {
      this.url = url;
      this.protocols = protocols;
      MockWebSocket.ultimaChiamata = { url, protocols };
    }

    close() {
      /* noop nel mock */
    }
  }

  beforeEach(() => {
    MockWebSocket.ultimaChiamata = null;
    vi.stubGlobal("WebSocket", MockWebSocket as unknown as typeof WebSocket);
  });

  afterEach(() => {
    syncFerma();
    vi.unstubAllGlobals();
  });

  it("la URL passata al costruttore WebSocket non contiene mai 'token='", async () => {
    await syncAvvia({
      serverUrl: "https://sync.example.com",
      email: "utente@esempio.it",
      token: "tok-jwt-xyz",
      intervalloSec: 3600,
      abilitato: true,
    });

    const chiamata = MockWebSocket.ultimaChiamata;
    expect(chiamata).not.toBeNull();
    expect(chiamata!.url).not.toContain("token=");
    expect(chiamata!.url).toBe("wss://sync.example.com/ws");
  });

  it("il token viaggia SOLO nel sub-protocollo, con il prefisso atteso dal server (#480)", async () => {
    await syncAvvia({
      serverUrl: "https://sync.example.com",
      email: "utente@esempio.it",
      token: "tok-jwt-xyz",
      intervalloSec: 3600,
      abilitato: true,
    });

    const chiamata = MockWebSocket.ultimaChiamata;
    expect(chiamata!.protocols).toEqual(["pap.sync.token.tok-jwt-xyz"]);
  });

  it("wss:// e' derivato correttamente anche se serverUrl usa HTTPS in maiuscolo", async () => {
    await syncAvvia({
      serverUrl: "HTTPS://sync.example.com",
      email: "utente@esempio.it",
      token: "tok-jwt-xyz",
      intervalloSec: 3600,
      abilitato: true,
    });

    const chiamata = MockWebSocket.ultimaChiamata;
    expect(chiamata!.url).toBe("wss://sync.example.com/ws");
  });
});
