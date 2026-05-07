<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Button from "./Button.svelte";
  import Input from "./Input.svelte";
  import Switch from "./Switch.svelte";
  import Toast from "./Toast.svelte";
  import Badge from "./Badge.svelte";

  interface ProviderConfigItem {
    provider: string;
    api_key?: string;
    base_url?: string | null;
    default_model?: string | null;
    abilitato: boolean;
    creato_a: string;
    aggiornato_a: string;
  }

  type ProviderKind =
    | "anthropic"
    | "openai"
    | "ollama"
    | "openai-compat"
    | "gemini";

  interface ProviderMeta {
    kind: ProviderKind;
    nome: string;
    icona: string;
    descrizione: string;
    placeholderModel: string;
    placeholderBaseUrl: string;
    richiedeApiKey: boolean;
  }

  const META: ProviderMeta[] = [
    {
      kind: "anthropic",
      nome: "Anthropic",
      icona: "🟠",
      descrizione: "Claude (Sonnet, Opus, Haiku) via console.anthropic.com",
      placeholderModel: "claude-sonnet-4-6",
      placeholderBaseUrl: "https://api.anthropic.com",
      richiedeApiKey: true,
    },
    {
      kind: "openai",
      nome: "OpenAI",
      icona: "🟢",
      descrizione: "GPT-4o, GPT-5 via platform.openai.com",
      placeholderModel: "gpt-4o",
      placeholderBaseUrl: "https://api.openai.com/v1",
      richiedeApiKey: true,
    },
    {
      kind: "ollama",
      nome: "Ollama",
      icona: "🦙",
      descrizione: "Modelli locali (llama3, mistral, ecc.)",
      placeholderModel: "llama3.2",
      placeholderBaseUrl: "http://localhost:11434",
      richiedeApiKey: false,
    },
    {
      kind: "openai-compat",
      nome: "OpenAI-compatibile",
      icona: "🔌",
      descrizione: "Endpoint compatibili (LM Studio, vLLM, OpenRouter)",
      placeholderModel: "modello-locale",
      placeholderBaseUrl: "http://localhost:1234/v1",
      richiedeApiKey: false,
    },
    {
      kind: "gemini",
      nome: "Google (Gemini)",
      icona: "🔷",
      descrizione: "Gemini (Flash, Pro) via aistudio.google.com/apikey",
      placeholderModel: "gemini-2.5-flash",
      placeholderBaseUrl: "https://generativelanguage.googleapis.com",
      richiedeApiKey: true,
    },
  ];

  let configs = $state<ProviderConfigItem[]>([]);
  let caricamento = $state(true);
  let erroreCaricamento = $state("");
  let provInModifica = $state<ProviderKind | null>(null);

  let formApiKey = $state("");
  let formBaseUrl = $state("");
  let formDefaultModel = $state("");
  let formAbilitato = $state(true);
  let formErrore = $state("");
  let formSalvataggio = $state(false);

  let toastVisibile = $state(false);
  let toastTesto = $state("");
  let toastVariante = $state<"success" | "danger">("success");

  let confermaElimina = $state(false);

  function showToast(testo: string, variante: "success" | "danger" = "success") {
    toastTesto = testo;
    toastVariante = variante;
    toastVisibile = true;
    setTimeout(() => (toastVisibile = false), 3000);
  }

  async function caricaConfigs() {
    caricamento = true;
    erroreCaricamento = "";
    try {
      configs = await invoke<ProviderConfigItem[]>("provider_config_lista");
    } catch (e) {
      erroreCaricamento = String(e);
    } finally {
      caricamento = false;
    }
  }

  function configEsistente(kind: ProviderKind): ProviderConfigItem | undefined {
    return configs.find((c) => c.provider === kind);
  }

  function apriForm(kind: ProviderKind) {
    const esistente = configEsistente(kind);
    provInModifica = kind;
    formApiKey = "";
    formBaseUrl = esistente?.base_url ?? "";
    formDefaultModel = esistente?.default_model ?? "";
    formAbilitato = esistente?.abilitato ?? true;
    formErrore = "";
    confermaElimina = false;
  }

  function chiudiForm() {
    provInModifica = null;
    formErrore = "";
    confermaElimina = false;
  }

  async function salvaForm() {
    if (provInModifica === null) return;
    formSalvataggio = true;
    formErrore = "";
    try {
      await invoke("provider_config_salva", {
        input: {
          provider: provInModifica,
          api_key: formApiKey || null,
          base_url: formBaseUrl || null,
          default_model: formDefaultModel || null,
          abilitato: formAbilitato,
        },
      });
      await caricaConfigs();
      showToast(`Config ${provInModifica} salvata`);
      chiudiForm();
    } catch (e) {
      formErrore = String(e);
    } finally {
      formSalvataggio = false;
    }
  }

  async function eliminaConfig() {
    if (provInModifica === null) return;
    formSalvataggio = true;
    formErrore = "";
    try {
      await invoke("provider_config_elimina", { provider: provInModifica });
      await caricaConfigs();
      showToast(`Config ${provInModifica} rimossa`);
      chiudiForm();
    } catch (e) {
      formErrore = String(e);
    } finally {
      formSalvataggio = false;
    }
  }

  $effect(() => {
    caricaConfigs();
  });

  const metaInModifica = $derived(
    provInModifica ? META.find((m) => m.kind === provInModifica) : null
  );
  const esistenteInModifica = $derived(
    provInModifica ? configEsistente(provInModifica) : undefined
  );
</script>

<div class="pannello">
  <h3 class="titolo">Provider AI</h3>
  <p class="desc">
    Configura i provider per eseguire i golden examples e il regression
    testing. Le API key sono salvate nel vault cifrato (SQLCipher AES-256)
    e non vengono mai inviate al frontend.
  </p>

  {#if caricamento}
    <div class="loading">Caricamento…</div>
  {:else if erroreCaricamento}
    <div class="errore">Errore: {erroreCaricamento}</div>
  {:else}
    <ul class="lista" role="list">
      {#each META as meta (meta.kind)}
        {@const cfg = configEsistente(meta.kind)}
        {@const configurato = !!cfg}
        <li class="card">
          <div class="card-head">
            <span class="card-icona" aria-hidden="true">{meta.icona}</span>
            <div class="card-info">
              <div class="card-nome">
                <span>{meta.nome}</span>
                {#if configurato && cfg?.abilitato}
                  <Badge variante="success">Attivo</Badge>
                {:else if configurato}
                  <Badge>Disabilitato</Badge>
                {:else}
                  <Badge>Non configurato</Badge>
                {/if}
              </div>
              <div class="card-desc">{meta.descrizione}</div>
              {#if configurato}
                <div class="card-meta">
                  {#if cfg?.default_model}
                    <span class="meta-item">
                      <span class="meta-label">Modello:</span>
                      <code>{cfg.default_model}</code>
                    </span>
                  {/if}
                  {#if cfg?.base_url}
                    <span class="meta-item">
                      <span class="meta-label">URL:</span>
                      <code>{cfg.base_url}</code>
                    </span>
                  {/if}
                </div>
              {/if}
            </div>
            <Button
              dimensione="sm"
              variante={configurato ? "ghost" : "primary"}
              onclick={() => apriForm(meta.kind)}
            >
              {configurato ? "Modifica" : "Configura"}
            </Button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if provInModifica && metaInModifica}
  <div
    class="modale-backdrop"
    onclick={chiudiForm}
    onkeydown={(e) => e.key === "Escape" && chiudiForm()}
    role="presentation"
  >
    <div
      class="modale"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modale-titolo"
      tabindex="-1"
    >
      <h4 id="modale-titolo" class="modale-titolo">
        {metaInModifica.icona}
        {esistenteInModifica ? "Modifica" : "Configura"} {metaInModifica.nome}
      </h4>

      {#if metaInModifica.richiedeApiKey}
        <div class="campo">
          <label for="api-key">API Key</label>
          <Input
            id="api-key"
            type="password"
            bind:valore={formApiKey}
            placeholder={esistenteInModifica
              ? "Lascia vuoto per non modificare"
              : "sk-…"}
            autocomplete="off"
          />
          <p class="hint">
            Salvata cifrata. Mai inviata al frontend dopo il save.
          </p>
        </div>
      {/if}

      <div class="campo">
        <label for="base-url"
          >Base URL {metaInModifica.richiedeApiKey ? "(opzionale)" : ""}</label
        >
        <Input
          id="base-url"
          bind:valore={formBaseUrl}
          placeholder={metaInModifica.placeholderBaseUrl}
        />
      </div>

      <div class="campo">
        <label for="default-model">Modello di default</label>
        <Input
          id="default-model"
          bind:valore={formDefaultModel}
          placeholder={metaInModifica.placeholderModel}
        />
      </div>

      <div class="campo campo-switch">
        <span class="switch-label">Abilitato</span>
        <Switch bind:attivo={formAbilitato} />
      </div>

      {#if formErrore}
        <div class="form-errore" role="alert">{formErrore}</div>
      {/if}

      <div class="modale-azioni">
        {#if esistenteInModifica}
          {#if !confermaElimina}
            <Button
              variante="ghost"
              dimensione="sm"
              onclick={() => (confermaElimina = true)}
            >
              Rimuovi configurazione
            </Button>
          {:else}
            <Button
              variante="danger"
              dimensione="sm"
              disabled={formSalvataggio}
              onclick={eliminaConfig}
            >
              Conferma rimozione
            </Button>
            <Button
              variante="ghost"
              dimensione="sm"
              onclick={() => (confermaElimina = false)}
            >
              Annulla
            </Button>
          {/if}
        {/if}
        <div class="azioni-spacer"></div>
        <Button
          variante="ghost"
          dimensione="sm"
          onclick={chiudiForm}
          disabled={formSalvataggio}
        >
          Annulla
        </Button>
        <Button
          variante="primary"
          dimensione="sm"
          disabled={formSalvataggio}
          onclick={salvaForm}
        >
          {formSalvataggio ? "Salvataggio…" : "Salva"}
        </Button>
      </div>
    </div>
  </div>
{/if}

<Toast visibile={toastVisibile} variante={toastVariante}>{toastTesto}</Toast>

<style>
  .pannello {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3, 1rem);
  }
  .titolo {
    margin: 0;
    font-size: var(--fs-base, 1.125rem);
    font-weight: var(--fw-semibold, 600);
  }
  .desc {
    margin: 0;
    color: var(--text-muted, #888);
    font-size: var(--fs-sm, 0.875rem);
    line-height: 1.5;
  }
  .lista {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2, 0.5rem);
  }
  .card {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    background: var(--bg-raised);
  }
  .card-head {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .card-icona {
    font-size: 1.5rem;
    line-height: 1;
  }
  .card-info {
    flex: 1;
    min-width: 0;
  }
  .card-nome {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: var(--fw-medium);
    color: var(--text-strong);
  }
  .card-desc {
    color: var(--text-muted);
    font-size: var(--fs-xs);
    margin-top: 2px;
  }
  .card-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 6px;
    font-size: var(--fs-xs);
  }
  .meta-item {
    color: var(--text-muted);
  }
  .meta-label {
    margin-right: 4px;
  }
  .meta-item code {
    background: var(--bg-overlay);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    color: var(--text-strong);
    font-family: var(--font-mono);
  }
  .loading,
  .errore {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
  }
  .errore {
    color: var(--danger);
  }
  .modale-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-modal, 1000);
  }
  .modale {
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 24px;
    width: min(28rem, 90vw);
    max-height: 90vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .modale-titolo {
    margin: 0;
    font-size: var(--fs-base);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }
  .campo {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .campo-switch {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
  .campo label,
  .switch-label {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-strong);
  }
  .hint {
    margin: 0;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }
  .form-errore {
    color: var(--danger);
    font-size: var(--fs-sm);
    padding: 8px 12px;
    background: var(--danger-soft);
    border-radius: var(--radius-sm);
  }
  .modale-azioni {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
  }
  .azioni-spacer {
    flex: 1;
  }
</style>
