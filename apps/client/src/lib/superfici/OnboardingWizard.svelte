<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    Badge,
    Button,
    Field,
    Input,
    Switch,
  } from "$lib/components";
  import StrengthMeter from "$lib/components/StrengthMeter.svelte";
  import ProfileCard from "$lib/components/ProfileCard.svelte";
  import HotkeyInput from "$lib/components/HotkeyInput.svelte";

  interface Props {
    oncompletato?: () => void;
  }

  let { oncompletato }: Props = $props();

  let step = $state(1);
  let profilo = $state<"personale" | "team">("personale");
  let password = $state("");
  let passwordConferma = $state("");
  let saltaCifratura = $state(false);
  let hotkey = $state("Ctrl+Shift+P");
  let creaPromptEsempio = $state(true);
  let tema = $state<"dark" | "light">("dark");

  let errore = $state("");
  let caricamento = $state(false);

  // Applica il tema al root document così l'utente vede subito
  // l'effetto del toggle, anche prima di completare l'onboarding.
  $effect(() => {
    document.documentElement.setAttribute("data-theme", tema);
  });

  const passwordValida = $derived(
    saltaCifratura || (password.length >= 8 && password === passwordConferma),
  );
  const passwordNonCorrispondono = $derived(
    !saltaCifratura &&
      passwordConferma.length > 0 &&
      password !== passwordConferma,
  );

  function avanti() {
    if (step === 2 && !passwordValida) {
      errore =
        !saltaCifratura && password.length < 8
          ? "La password deve avere almeno 8 caratteri"
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

  async function saltaTour() {
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
          profilo,
          hotkey,
          tema,
          tono: "zinc",
          lingua: "it",
          onboarding_completato: true,
          crea_prompt_esempio: creaPromptEsempio,
        },
      });

      await invoke("registra_hotkey", { combo: hotkey });
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
          Una libreria locale per i prompt che usi ogni giorno. Scegli come vuoi
          iniziare — potrai cambiare in qualsiasi momento.
        </p>

        <div class="profilo-grid">
          <ProfileCard
            titolo="Personale"
            descrizione="Vault locale, zero configurazione"
            dettagli={[
              "I prompt vivono solo sul tuo PC",
              "Nessun account, nessun server",
              "Cifratura locale opzionale",
            ]}
            selezionato={profilo === "personale"}
            variante="private"
            onclick={() => (profilo = "personale")}
          >
            {#snippet icona()}
              <svg
                width="20"
                height="20"
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
            {/snippet}
          </ProfileCard>

          <ProfileCard
            titolo="Team"
            descrizione="Sync con il server del tuo workspace"
            dettagli={[
              "Condividi prompt con ruoli e permessi",
              "I prompt privati restano locali",
              "Server self-hosted o gestito",
            ]}
            variante="team"
            selezionato={profilo === "team"}
            onclick={() => (profilo = "team")}
          >
            {#snippet icona()}
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
                <circle cx="9" cy="7" r="4" />
                <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
                <path d="M16 3.13a4 4 0 0 1 0 7.75" />
              </svg>
            {/snippet}
          </ProfileCard>
        </div>
      {:else if step === 2}
        <h1 class="wizard-titolo">Cifra il tuo vault</h1>
        <p class="wizard-desc">
          I tuoi prompt privati saranno cifrati con AES-256. La password non
          viene mai trasmessa né recuperabile — annotala in un password manager.
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
              >Crea un primo prompt di esempio</strong
            >
            <p class="subtle">
              Aggiungiamo "Riassunto bug report" alla tua libreria — puoi
              cancellarlo subito se vuoi.
            </p>
          </div>
          <Switch
            bind:attivo={creaPromptEsempio}
            etichetta="Crea prompt di esempio al primo avvio"
            privato
          />
        </div>
      {/if}
    </div>

    {#if errore}
      <p class="wizard-errore">{errore}</p>
    {/if}

    <div class="wizard-footer">
      {#if step === 1}
        <Button variante="ghost" onclick={saltaTour} disabled={caricamento}>
          Salta tour
        </Button>
      {:else}
        <Button variante="ghost" onclick={indietro} disabled={caricamento}>
          ← Indietro
        </Button>
      {/if}

      {#if step < 3}
        <Button variante="primary" onclick={avanti}>Continua →</Button>
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
    background: var(--accent-team);
  }

  .progress-step--attivo {
    background: var(--accent-team);
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

  .profilo-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-3);
    margin-top: var(--sp-2);
  }

  .password-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    max-width: 400px;
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
</style>
