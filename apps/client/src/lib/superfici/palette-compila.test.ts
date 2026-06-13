/**
 * Test per la logica di espansione {{import "..."}} nella palette.
 *
 * Issue #299: compilaECopia non espandeva i token {{import}} prima della copia.
 * Fix: specchia il pattern di CompilaModal (issue #293) — se il body contiene
 * `{{import "`, chiama prompt_compila_inline (backend) prima di passare il body
 * a compila(). I test coprono la logica pura via `palette-espansione.ts`:
 * - (a) body senza import → nessuna invoke, ritorna raw
 * - (b) body con import → invoke chiamata, ritorna espanso
 * - (c) invoke lancia errore → fallback raw + erroreEspansione
 * - (d) out-of-order → risposta tardiva scartata (non sovrascrive la nuova)
 * - (e) guard tight: {{importanza}} non viene considerato import (MEDIUM-2)
 */

import { describe, it, expect, vi } from "vitest";
import { compila, estraiSegnaposti } from "$lib/template";
import {
  contieneImport,
  espandiImportConToken,
} from "$lib/util/palette-espansione";

// ── Costanti di test ─────────────────────────────────────────────────────────

const BODY_SENZA_IMPORT = "Ciao {{nome}}, benvenuto in {{luogo}}!";
const BODY_CON_IMPORT = '{{import "intro/saluto"}} Ciao {{nome}}!';
const BODY_ESPANSO_SIMULATO = "Benvenuto nel team! Ciao {{nome}}!";
const BODY_CON_IMPORTANZA = "Questo ha {{importanza}} alta";

// ── (a) Guard: contieneImport ─────────────────────────────────────────────────

describe("contieneImport: rilevamento token {{import \"}} (MEDIUM-2 tightened guard)", () => {
  it("ritorna false per body senza token import", () => {
    expect(contieneImport(BODY_SENZA_IMPORT)).toBe(false);
  });

  it('ritorna true per body con {{import "path"}}', () => {
    expect(contieneImport(BODY_CON_IMPORT)).toBe(true);
  });

  it('ritorna false per {{importanza}} — guard tightened, non è un import (MEDIUM-2)', () => {
    expect(contieneImport(BODY_CON_IMPORTANZA)).toBe(false);
  });

  it("ritorna false per stringa vuota", () => {
    expect(contieneImport("")).toBe(false);
  });
});

// ── (b) body con import → invoke chiamata, ritorna espanso ───────────────────

describe("espandiImportConToken: caso normale (invoke ok)", () => {
  it("chiama invokeFn e ritorna bodyEspanso quando il token è aggiornato", async () => {
    // Arrange
    let seq = 1;
    const invokeFn = vi.fn().mockResolvedValue(BODY_ESPANSO_SIMULATO);

    // Act
    const risultato = await espandiImportConToken(
      BODY_CON_IMPORT,
      "prompt-123",
      invokeFn,
      seq,
      () => seq,
    );

    // Assert
    expect(invokeFn).toHaveBeenCalledOnce();
    expect(invokeFn).toHaveBeenCalledWith(BODY_CON_IMPORT, "prompt-123");
    expect(risultato).not.toBeNull();
    expect(risultato!.bodyEspanso).toBe(BODY_ESPANSO_SIMULATO);
    expect(risultato!.erroreEspansione).toBeNull();
  });
});

// ── (a) body senza import → nessuna invoke ───────────────────────────────────

describe("espandiImportConToken: body senza import", () => {
  it("non chiama invokeFn e ritorna bodyEspanso=null, erroreEspansione=null", async () => {
    // Arrange
    let seq = 1;
    const invokeFn = vi.fn();

    // Act
    const risultato = await espandiImportConToken(
      BODY_SENZA_IMPORT,
      "prompt-123",
      invokeFn,
      seq,
      () => seq,
    );

    // Assert
    expect(invokeFn).not.toHaveBeenCalled();
    expect(risultato).not.toBeNull();
    expect(risultato!.bodyEspanso).toBeNull();
    expect(risultato!.erroreEspansione).toBeNull();
  });
});

// ── (c) invoke lancia errore → fallback + erroreEspansione ───────────────────

describe("espandiImportConToken: invoke fallisce", () => {
  it("ritorna bodyEspanso=null e erroreEspansione con messaggio pulito", async () => {
    // Arrange: il backend Tauri lancia un errore come stringa con prefisso "Error: "
    let seq = 1;
    const invokeFn = vi
      .fn()
      .mockRejectedValue("Error: cycle detected");

    // Act
    const risultato = await espandiImportConToken(
      BODY_CON_IMPORT,
      "prompt-123",
      invokeFn,
      seq,
      () => seq,
    );

    // Assert
    expect(risultato).not.toBeNull();
    expect(risultato!.bodyEspanso).toBeNull();
    // "Error: " prefix rimosso tramite replace (come nel componente originale)
    expect(risultato!.erroreEspansione).toBe("cycle detected");
  });
});

// ── (d) out-of-order: risposta tardiva scartata (HIGH-1) ─────────────────────

describe("espandiImportConToken: out-of-order / race condition (HIGH-1)", () => {
  it("scarta la risposta di A quando B ha già incrementato il token", async () => {
    // Arrange: simula il controller del componente
    let seqGlobale = 0;

    // invokeFn per A (lenta) e B (veloce)
    let resolveA!: (v: string) => void;
    const invokeA = vi.fn(
      () => new Promise<string>((res) => (resolveA = res)),
    );
    const invokeB = vi.fn().mockResolvedValue("espanso-B");

    // Act: avvia A (token=1)
    seqGlobale = 1;
    const tokenA = seqGlobale;
    const promessaA = espandiImportConToken(
      BODY_CON_IMPORT,
      "prompt-A",
      invokeA,
      tokenA,
      () => seqGlobale,
    );

    // Poi avvia B (token=2) — sovrascrive il contatore
    seqGlobale = 2;
    const tokenB = seqGlobale;
    const risultatoB = await espandiImportConToken(
      BODY_CON_IMPORT,
      "prompt-B",
      invokeB,
      tokenB,
      () => seqGlobale,
    );

    // Risolve A in ritardo (dopo che B è già completata)
    resolveA("espanso-A-tardivo");
    const risultatoA = await promessaA;

    // Assert: B è ok, A è scartata (null = fuori-ordine)
    expect(risultatoB).not.toBeNull();
    expect(risultatoB!.bodyEspanso).toBe("espanso-B");

    // A restituisce null perché token 1 !== seqGlobale 2 quando risolve
    expect(risultatoA).toBeNull();
  });

  it("scarta anche l'errore fuori-ordine di A", async () => {
    // Arrange
    let seqGlobale = 0;

    let rejectA!: (e: unknown) => void;
    const invokeA = vi.fn(
      () => new Promise<string>((_, rej) => (rejectA = rej)),
    );
    const invokeB = vi.fn().mockResolvedValue("espanso-B");

    // Avvia A (token=1)
    seqGlobale = 1;
    const promessaA = espandiImportConToken(
      BODY_CON_IMPORT,
      "prompt-A",
      invokeA,
      1,
      () => seqGlobale,
    );

    // Avvia B (token=2)
    seqGlobale = 2;
    const risultatoB = await espandiImportConToken(
      BODY_CON_IMPORT,
      "prompt-B",
      invokeB,
      2,
      () => seqGlobale,
    );

    // A rigetta in ritardo
    rejectA(new Error("timeout"));
    const risultatoA = await promessaA;

    expect(risultatoB!.bodyEspanso).toBe("espanso-B");
    expect(risultatoA).toBeNull(); // scartato
  });
});

// ── Integrazione template: compilazione sul body espanso ────────────────────

describe("palette: compilazione sul body espanso (integrazione template)", () => {
  it("compila i segnaposti sul body espanso invece del raw", () => {
    // Arrange
    const valori = { nome: "Giulia" };

    // Act
    const risultatoEspanso = compila(BODY_ESPANSO_SIMULATO, valori);
    const risultatoRaw = compila(BODY_CON_IMPORT, valori);

    // Assert: con body espanso il token import è risolto
    expect(risultatoEspanso).toBe("Benvenuto nel team! Ciao Giulia!");
    expect(risultatoEspanso).not.toContain("{{import");

    // Prima del fix il raw conteneva ancora {{import}} non risolto
    expect(risultatoRaw).toContain('{{import "intro/saluto"}}');
  });

  it("estrae segnaposti dal body espanso (import può introdurre nuovi segnaposti)", () => {
    // Arrange
    const bodyEspansoConNuoviSeg = "Ciao {{nome}}, sei il nuovo {{ruolo}}!";

    // Act
    const segDaEspanso = estraiSegnaposti(bodyEspansoConNuoviSeg);
    const segDaRaw = estraiSegnaposti(BODY_CON_IMPORT);

    // Assert: il form segnaposti deve derivare dall'espanso per mostrare tutti i campi
    expect(segDaEspanso.map((s) => s.nome)).toContain("ruolo");
    expect(segDaRaw.map((s) => s.nome)).not.toContain("ruolo");
  });

  it("usa il body raw come fallback quando bodyEspanso è null", () => {
    // Arrange
    const bodyEspanso: string | null = null;
    const valori = { nome: "Marco", luogo: "Roma" };

    // Act: bodyPerCompilazione = bodyEspanso ?? rawBody (pattern CompilaModal)
    const bodyPerCompilazione = bodyEspanso ?? BODY_SENZA_IMPORT;
    const risultato = compila(bodyPerCompilazione, valori);

    // Assert
    expect(risultato).toBe("Ciao Marco, benvenuto in Roma!");
  });
});
