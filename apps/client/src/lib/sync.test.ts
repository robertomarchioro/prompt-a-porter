import { describe, it, expect } from "vitest";
import { validaServerUrl } from "./sync";

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
