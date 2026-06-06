<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { enable as autostartEnable } from "@tauri-apps/plugin-autostart";
  import {
    Badge,
    Button,
    Field,
    Input,
    Switch,
  } from "$lib/components";
  import StrengthMeter from "$lib/components/StrengthMeter.svelte";
  import HotkeyInput from "$lib/components/HotkeyInput.svelte";
  import Modale from "$lib/components/Modale.svelte";
  import demoVault from "../../../../../docs/demo/demo-vault.json";

  // Forma del risultato di vault_import_json (mirrors import_export.rs::ImportReport).
  interface ImportReport {
    nuovi: number;
    aggiornati: number;
    conflitti: number;
    errori: string[];
  }

  interface Props {
    oncompletato?: () => void;
  }

  let { oncompletato }: Props = $props();

  let step = $state(1);
  let password = $state("");
  let passwordConferma = $state("");
  let saltaCifratura = $state(false);
  let hotkey = $state("Ctrl+Shift+P");
  let creaPromptEsempio = $state(true);
  // Issue #282: avvio automatico con Windows — OFF di default.
  let avvioAutomatico = $state(false);
  let avvioPortable = $state(false);
  // Issue #269: tema light di default al primo avvio.
  let tema = $state<"dark" | "light">("light");

  // Issue #281: modale di conferma per "Salta tour".
  let confermaSaltaTourAperta = $state(false);

  // Issue #270: criterio reale lato backend (vault.rs PASSWORD_MIN_LEN = 8).
  const PASSWORD_MIN_LEN = 8;

  let errore = $state("");
  let caricamento = $state(false);

  // Applica il tema al root document così l'utente vede subito
  // l'effetto del toggle, anche prima di completare l'onboarding.
  $effect(() => {
    document.documentElement.setAttribute("data-theme", tema);
  });

  // Issue #282: rileva la modalità portable una volta sola quando il componente
  // viene montato, così da nascondere il toggle avvio automatico se non supportato.
  $effect(() => {
    invoke<boolean>("app_is_portable")
      .then((v) => {
        avvioPortable = v;
      })
      .catch(() => {
        avvioPortable = false;
      });
  });

  // Issue #270: criteri di complessità esplicitati e validati inline.
  const criterioLunghezza = $derived(password.length >= PASSWORD_MIN_LEN);
  const criterioCorrispondenza = $derived(
    password.length > 0 && password === passwordConferma,
  );
  const passwordValida = $derived(
    saltaCifratura || (criterioLunghezza && criterioCorrispondenza),
  );
  const passwordNonCorrispondono = $derived(
    !saltaCifratura &&
      passwordConferma.length > 0 &&
      password !== passwordConferma,
  );

  function avanti() {
    if (step === 2 && !passwordValida) {
      errore =
        !saltaCifratura && password.length < PASSWORD_MIN_LEN
          ? `La password deve avere almeno ${PASSWORD_MIN_LEN} caratteri`
          : "Le password non corrispondono";
      return;
    }
    errore = "";
    if (step < 3) step++;
  }

  function indietro() {
    errore = "";
    if (step > 1) step--;
  }

  // Issue #281: mostra la modale di conferma invece di saltare subito.
  function apriConfermaSaltaTour() {
    confermaSaltaTourAperta = true;
  }

  async function saltaTour() {
    confermaSaltaTourAperta = false;
    caricamento = true;
    errore = "";
    try {
      await invoke("vault_crea_aperto");
      await invoke("preferenze_salva", {
        preferenze: {
          profilo: "personale",
          hotkey: "Ctrl+Shift+P",
          tema,
          tono: "zinc",
          lingua: "it",
          onboarding_completato: true,
          crea_prompt_esempio: false,
        },
      });
      await invoke("registra_hotkey", { combo: "Ctrl+Shift+P" });
      // Issue #282: "Salta tour" lascia l'avvio automatico disattivato (default OFF).
      oncompletato?.();
    } catch (e) {
      errore = String(e);
      caricamento = false;
    }
  }

  async function completa() {
    caricamento = true;
    errore = "";
    try {
      if (saltaCifratura) {
        await invoke("vault_crea_aperto");
      } else {
        await invoke("vault_crea", { password });
      }

      await invoke("preferenze_salva", {
        preferenze: {
          profilo: "personale",
          hotkey,
          tema,
          tono: "zinc",
          lingua: "it",
          onboarding_completato: true,
          crea_prompt_esempio: creaPromptEsempio,
        },
      });

      await invoke("registra_hotkey", { combo: hotkey });

      // Issue #284: importa il demo vault invece di creare un singolo
      // prompt hardcoded. Un fallimento qui non deve bloccare l'onboarding.
      if (creaPromptEsempio) {
        try {
          const report = await invoke<ImportReport>("vault_import_json", {
            json: JSON.stringify(demoVault),
            modalita: "skip",
          });
          if (report.errori.length > 0) {
            console.warn(
              "[onboarding] import demo parziale — alcuni elementi ignorati:",
              report.errori,
            );
          }
        } catch (errEsempio) {
          console.error(
            "[onboarding] importazione demo vault fallita",
            errEsempio,
          );
        }
      }

      // Issue #282: se l'utente ha abilitato l'avvio automatico e non siamo in
      // modalità portable, attiva il plugin. Non bloccante: un errore qui non
      // deve impedire il completamento dell'onboarding.
      if (avvioAutomatico && !avvioPortable) {
        try {
          await autostartEnable();
        } catch (errAvvio) {
          console.error(
            "[onboarding] attivazione avvio automatico fallita",
            errAvvio,
          );
        }
      }

      oncompletato?.();
    } catch (e) {
      errore = String(e);
      caricamento = false;
    }
  }

  function gestisciTastiera(e: KeyboardEvent) {
    if (
      e.key === "Enter" &&
      !e.ctrlKey &&
      !e.shiftKey &&
      !e.altKey &&
      !e.metaKey &&
      !caricamento
    ) {
      e.preventDefault();
      if (step < 3) avanti();
      else completa();
    }
  }
</script>

<svelte:window onkeydown={gestisciTastiera} />

<div class="wizard-overlay">
  <div class="wizard">
    <div class="wizard-top">
      <div class="progress">
        {#each [1, 2, 3] as s}
          <div
            class="progress-step"
            class:progress-step--completato={s < step}
            class:progress-step--attivo={s === step}
          ></div>
        {/each}
      </div>
      <div class="tema-toggle" role="group" aria-label="Tema">
        <button
          type="button"
          class="tema-opzione"
          class:tema-opzione--attivo={tema === "dark"}
          aria-pressed={tema === "dark"}
          onclick={() => (tema = "dark")}
        >Scuro</button>
        <button
          type="button"
          class="tema-opzione"
          class:tema-opzione--attivo={tema === "light"}
          aria-pressed={tema === "light"}
          onclick={() => (tema = "light")}
        >Chiaro</button>
      </div>
    </div>

    <div class="wizard-content">
      {#if step === 1}
        <h1 class="wizard-titolo">Benvenuto in Prompt a Porter</h1>
        <p class="wizard-desc">
          La tua libreria locale per i prompt AI. I tuoi prompt vivono solo
          sul tuo PC — nessun account, nessun server, zero configurazione.
          Configuriamo insieme le opzioni essenziali in tre passi.
        </p>
        <div class="benvenuto-card">
          <div class="benvenuto-icona" aria-hidden="true">
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
          </div>
          <ul class="benvenuto-lista">
            <li>I prompt vivono solo sul tuo PC</li>
            <li>Nessun account, nessun server</li>
            <li>Cifratura locale opzionale</li>
          </ul>
        </div>
      {:else if step === 2}
        <h1 class="wizard-titolo">Cifra il tuo vault</h1>
        <p class="wizard-desc">
          I tuoi prompt privati saranno cifrati con AES-256. La password non
          viene mai trasmessa né recuperabile — annotala in un password manager.
          Deve avere almeno {PASSWORD_MIN_LEN} caratteri.
        </p>

        {#if !saltaCifratura}
          <div class="password-form">
            <Field etichetta="Password">
              <Input
                bind:valore={password}
                type="password"
                placeholder="Password del vault"
                invalido={errore !== "" && password.length < 8}
              />
            </Field>
            <StrengthMeter {password} />
            <ul class="criteri" aria-label="Criteri password">
              <li
                class="criterio"
                class:criterio--ok={criterioLunghezza}
                aria-label={`Almeno ${PASSWORD_MIN_LEN} caratteri: ${
                  criterioLunghezza ? "soddisfatto" : "non soddisfatto"
                }`}
              >
                <span class="criterio-icona" aria-hidden="true"
                  >{criterioLunghezza ? "✓" : "○"}</span
                >
                <span aria-hidden="true">Almeno {PASSWORD_MIN_LEN} caratteri</span
                >
              </li>
              <li
                class="criterio"
                class:criterio--ok={criterioCorrispondenza}
                aria-label={`Le due password coincidono: ${
                  criterioCorrispondenza ? "soddisfatto" : "non soddisfatto"
                }`}
              >
                <span class="criterio-icona" aria-hidden="true"
                  >{criterioCorrispondenza ? "✓" : "○"}</span
                >
                <span aria-hidden="true">Le due password coincidono</span>
              </li>
            </ul>
            <Field
              etichetta="Conferma password"
              errore={passwordNonCorrispondono
                ? "Le password non corrispondono"
                : ""}
            >
              <Input
                bind:valore={passwordConferma}
                type="password"
                placeholder="Conferma password"
                invalido={passwordNonCorrispondono}
              />
            </Field>
          </div>
        {/if}

        <label class="skip-cifratura">
          <Switch bind:attivo={saltaCifratura} etichetta="Salta cifratura del vault" />
          <span>Salta cifratura — il vault sarà in chiaro sul disco.</span>
          <Badge variante="warning">Sconsigliato</Badge>
        </label>
      {:else if step === 3}
        <h1 class="wizard-titolo">Imposta la tua hotkey globale</h1>
        <p class="wizard-desc">
          La scorciatoia per evocare il Command Palette da qualunque app. Premi
          una combinazione per registrarla.
        </p>

        <HotkeyInput bind:valore={hotkey} />

        <div class="esempio-prompt">
          <div class="esempio-testo">
            <strong class="esempio-titolo"
              >Importa i prompt di esempio</strong
            >
            <p class="subtle">
              Aggiungiamo una raccolta di prompt pronti all'uso alla tua
              libreria — puoi cancellarli in qualsiasi momento.
            </p>
          </div>
          <Switch
            bind:attivo={creaPromptEsempio}
            etichetta="Importa prompt di esempio al primo avvio"
            privato
          />
        </div>

        {#if !avvioPortable}
          <div class="avvio-automatico">
            <div class="esempio-testo">
              <strong class="esempio-titolo">Avvia con Windows</strong>
              <p class="subtle">
                Prompt a Porter parte in background all'avvio del PC — la hotkey
                è subito disponibile. Puoi disattivarlo in Impostazioni.
              </p>
            </div>
            <Switch
              bind:attivo={avvioAutomatico}
              etichetta="Avvia automaticamente con Windows"
              privato
            />
          </div>
        {/if}
      {/if}
    </div>

    {#if errore}
      <p class="wizard-errore">{errore}</p>
    {/if}

    <div class="wizard-footer">
      {#if step === 1}
        <Button variante="ghost" onclick={apriConfermaSaltaTour} disabled={caricamento}>
          Salta tour
        </Button>
      {:else}
        <Button variante="ghost" onclick={indietro} disabled={caricamento}>
          ← Indietro
        </Button>
      {/if}

      {#if step < 3}
        <Button
          variante="primary"
          onclick={avanti}
          disabled={step === 2 && !passwordValida}
        >Continua →</Button>
      {:else}
        <Button
          variante="primary"
          dimensione="lg"
          onclick={completa}
          disabled={caricamento}
        >
          {caricamento ? "Creazione vault…" : "Inizia ad usare Prompt a Porter"}
        </Button>
      {/if}
    </div>

    <div class="wizard-hint">
      <kbd>⏎</kbd> continua
    </div>
  </div>
</div>

<!-- Issue #281: conferma prima di saltare il tour. -->
{#if confermaSaltaTourAperta}
  <Modale
    titolo="Salta la configurazione guidata?"
    larghezza="sm"
    onChiudi={() => (confermaSaltaTourAperta = false)}
  >
    <p class="salta-desc">
      Saltando il tour verranno applicati questi valori predefiniti:
    </p>
    <ul class="salta-lista">
      <li><strong>Profilo:</strong> Personale (vault locale)</li>
      <li><strong>Scorciatoia:</strong> Ctrl+Shift+P</li>
      <li><strong>Prompt di esempio:</strong> nessuno importato</li>
    </ul>
    <p class="salta-nota">Potrai modificare queste impostazioni in qualsiasi momento.</p>
    {#snippet footer()}
      <Button variante="ghost" onclick={() => (confermaSaltaTourAperta = false)}>
        Annulla
      </Button>
      <Button variante="primary" onclick={saltaTour} disabled={caricamento}>
        Confermo, salta
      </Button>
    {/snippet}
  </Modale>
{/if}

<style>
  .wizard-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-canvas);
    font-family: var(--font-ui);
    z-index: var(--z-modal);
  }

  .wizard {
    width: 720px;
    max-width: 95vw;
    max-height: 90vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--sp-5);
  }

  .wizard-top {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .progress {
    display: flex;
    gap: 3px;
    flex: 1;
  }

  .tema-toggle {
    display: inline-flex;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    padding: 2px;
    flex-shrink: 0;
  }

  .tema-opzione {
    appearance: none;
    background: transparent;
    border: none;
    padding: 4px 12px;
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-full);
    transition: background var(--motion-fast) var(--easing-standard),
      color var(--motion-fast) var(--easing-standard);
  }

  .tema-opzione:hover {
    color: var(--text-default);
  }

  .tema-opzione--attivo {
    background: var(--bg-canvas);
    color: var(--text-strong);
    box-shadow: var(--shadow-1);
  }

  .progress-step {
    height: 4px;
    flex: 1;
    background: var(--border-subtle);
    border-radius: var(--radius-full);
    transition: background var(--motion-normal) var(--easing-standard);
  }

  .progress-step--completato {
    background: var(--accent-private);
  }

  .progress-step--attivo {
    background: var(--accent-private);
    animation: pulse 1.6s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }

  .wizard-content {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .wizard-titolo {
    font-size: var(--fs-2xl);
    font-weight: var(--fw-bold);
    color: var(--text-strong);
    letter-spacing: var(--tracking-tight);
    margin: 0;
  }

  .wizard-desc {
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: var(--lh-relaxed);
    margin: 0;
    max-width: 560px;
  }

  .benvenuto-card {
    display: flex;
    align-items: flex-start;
    gap: var(--sp-4);
    padding: var(--sp-4);
    background: var(--accent-private-soft);
    border-radius: var(--radius-lg);
    margin-top: var(--sp-2);
  }

  .benvenuto-icona {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent-private-soft);
    border-radius: var(--radius-md);
    color: var(--accent-private);
    flex-shrink: 0;
  }

  .benvenuto-lista {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .benvenuto-lista li {
    font-size: var(--fs-sm);
    color: var(--text-muted);
    padding-left: var(--sp-3);
    position: relative;
  }

  .benvenuto-lista li::before {
    content: "•";
    position: absolute;
    left: var(--sp-1);
    color: var(--accent-private);
  }

  .password-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    max-width: 400px;
  }

  .criteri {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .criterio {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .criterio--ok {
    color: var(--accent-private, var(--text-default));
  }

  .criterio-icona {
    display: inline-flex;
    width: 1em;
    justify-content: center;
    font-weight: var(--fw-bold);
  }

  .skip-cifratura {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--text-muted);
    cursor: pointer;
    margin-top: var(--sp-2);
  }

  .esempio-prompt {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-4);
    background: var(--accent-private-soft);
    border-radius: var(--radius-lg);
    margin-top: var(--sp-2);
  }

  /* Issue #282: box avvio automatico — usa --accent-team-soft (viola/indigo)
     per distinguersi dal box esempio-prompt che usa --accent-private-soft (ambra). */
  .avvio-automatico {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-4);
    background: var(--accent-team-soft);
    border-radius: var(--radius-lg);
  }

  .esempio-testo {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .esempio-titolo {
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .wizard-errore {
    font-size: var(--fs-sm);
    color: var(--danger);
    margin: 0;
  }

  .wizard-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: var(--sp-3);
    border-top: 1px solid var(--border-subtle);
  }

  .wizard-hint {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-1);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  .wizard-hint kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
  }

  /* Issue #281: stili per il contenuto della modale "Salta tour". */
  .salta-desc {
    font-size: var(--fs-sm);
    color: var(--text-default);
    margin: 0 0 var(--sp-2);
  }

  .salta-lista {
    list-style: none;
    padding: 0;
    margin: 0 0 var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .salta-lista li {
    font-size: var(--fs-sm);
    color: var(--text-muted);
    padding-left: var(--sp-3);
    position: relative;
  }

  .salta-lista li::before {
    content: "•";
    position: absolute;
    left: var(--sp-1);
    color: var(--accent-private);
  }

  .salta-nota {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    margin: 0;
  }
</style>
