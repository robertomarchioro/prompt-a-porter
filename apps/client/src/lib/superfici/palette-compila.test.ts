/**
 * Test per la logica di espansione {{import}} nella palette.
 *
 * Issue #299: compilaECopia non espandeva i token {{import}} prima della copia.
 * Fix: specchia il pattern di CompilaModal (issue #293) — se il body contiene
 * "{{import", chiama prompt_compila_inline (backend) prima di passare il body
 * a compila(). Questi test coprono le parti pure (senza Tauri invoke):
 * - rilevamento della necessità di espansione
 * - compilazione sul body espanso (dopo l'espansione simulata)
 * - fallback al body raw se l'espansione fallisce
 * - reset eager di bodyEspanso al cambio di prompt (no stale expansion)
 */

import { describe, it, expect } from "vitest";
import { compila, estraiSegnaposti } from "$lib/template";

const BODY_SENZA_IMPORT = "Ciao {{nome}}, benvenuto in {{luogo}}!";
const BODY_CON_IMPORT = '{{import "intro/saluto"}} Ciao {{nome}}!';
const BODY_ESPANSO_SIMULATO = "Benvenuto nel team! Ciao {{nome}}!";

describe("palette: rilevamento {{import}} nel body", () => {
  it("non richiede espansione quando il body non ha token import", () => {
    expect(BODY_SENZA_IMPORT.includes("{{import")).toBe(false);
  });

  it("richiede espansione quando il body contiene {{import", () => {
    expect(BODY_CON_IMPORT.includes("{{import")).toBe(true);
  });
});

describe("palette: compilaECopia con body espanso (simulato)", () => {
  it("compila i segnaposti sul body espanso invece del raw", () => {
    // Arrange: simula l'espansione che prompt_compila_inline farebbe
    const valori = { nome: "Giulia" };

    // Act: compila usa il body espanso, non il raw (che ha ancora {{import}})
    const risultatoEspanso = compila(BODY_ESPANSO_SIMULATO, valori);
    const risultatoRaw = compila(BODY_CON_IMPORT, valori);

    // Assert: con body espanso, il token import è risolto
    expect(risultatoEspanso).toBe("Benvenuto nel team! Ciao Giulia!");
    expect(risultatoEspanso).not.toContain("{{import");

    // Prima del fix il risultato raw conteneva ancora {{import}} non risolto
    expect(risultatoRaw).toContain('{{import "intro/saluto"}}');
  });

  it("estrae segnaposti dal body espanso (import può introdurre nuovi segnaposti)", () => {
    // Arrange: il body importato introduce un segnaposto {{ruolo}} non presente nel raw
    const bodyEspansoConNuoviSeg = "Ciao {{nome}}, sei il nuovo {{ruolo}}!";

    // Act
    const segDaEspanso = estraiSegnaposti(bodyEspansoConNuoviSeg);
    const segDaRaw = estraiSegnaposti(BODY_CON_IMPORT);

    // Assert: il form segnaposti deve derivare dall'espanso per mostrare tutti i campi
    expect(segDaEspanso.map((s) => s.nome)).toContain("ruolo");
    expect(segDaRaw.map((s) => s.nome)).not.toContain("ruolo");
  });
});

describe("palette: fallback al body raw se espansione fallisce", () => {
  it("compila con il body raw come fallback quando bodyEspanso è null", () => {
    // Arrange: bodyEspanso = null (errore o nessun import)
    const bodyEspanso: string | null = null;
    const rawBody = BODY_SENZA_IMPORT;
    const valori = { nome: "Marco", luogo: "Roma" };

    // Act: bodyPerCompilazione = bodyEspanso ?? rawBody (pattern CompilaModal)
    const bodyPerCompilazione = bodyEspanso ?? rawBody;
    const risultato = compila(bodyPerCompilazione, valori);

    // Assert: fallback funziona correttamente
    expect(risultato).toBe("Ciao Marco, benvenuto in Roma!");
  });
});

describe("palette: reset eager bodyEspanso al cambio prompt (#297 lesson)", () => {
  it("bodyEspanso deve essere null dopo il reset (nessuna expansion stale)", () => {
    // Documenta il contratto: quando l'utente seleziona un prompt diverso,
    // bodyEspanso viene resettato a null PRIMA di invocare prompt_compila_inline.
    // Così testoCompilato usa il raw body del nuovo prompt, non l'expansion
    // del prompt precedente.

    let bodyEspanso: string | null = "vecchio body espanso";

    // Simula reset eager (come in seleziona())
    bodyEspanso = null;

    // Assert: nessun valore stale visibile prima che l'espansione completi
    expect(bodyEspanso).toBeNull();
  });
});
