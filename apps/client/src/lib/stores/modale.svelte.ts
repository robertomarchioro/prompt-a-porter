/**
 * Store globale Svelte 5 per gestire la modale attiva (V0.8 F8).
 *
 * Discriminated union: ogni tipo modale specifica il payload contestuale
 * necessario (es. promptId per Compila). Una sola modale attiva alla volta.
 * Ephemeral — nessuna persistenza tra reload.
 *
 * Riferimenti:
 * - Blueprint: docs/roadmap/redesign-v08/blueprint-F8.md §1
 */

export type ModaleAttiva =
  | { tipo: "compila"; promptId: string }
  | { tipo: "insight" }
  | { tipo: "regressioni" }
  | { tipo: "impostazioni"; sezione?: string }
  | { tipo: "palette" }
  | null;

class StatoModale {
  attiva = $state<ModaleAttiva>(null);
}

export const statoModale = new StatoModale();

export function apriModale(modale: NonNullable<ModaleAttiva>): void {
  statoModale.attiva = modale;
}

export function chiudiModale(): void {
  statoModale.attiva = null;
}
