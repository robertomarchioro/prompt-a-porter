<script lang="ts">
  import {
    Badge,
    Button,
    EmptyState,
    Field,
    Input,
    Kbd,
    NavItem,
    Placeholder,
    Select,
    Skeleton,
    Switch,
    Tag,
    Textarea,
    Toast,
    Tooltip,
  } from "$lib/components";

  let inputValore = $state("Testo di esempio");
  let textareaValore = $state("Scrivi il tuo prompt qui…\nUsa {{nome}} per i parametri.");
  let selectValore = $state("claude");
  let switchAttivo = $state(false);
  let switchPrivato = $state(true);
  let toastVisibile = $state(false);

  let temaCorrente = $state<"dark" | "light">("dark");
  let tonoCorrente = $state<"zinc" | "slate" | "stone">("zinc");

  function cambiaTema(tema: "dark" | "light") {
    temaCorrente = tema;
    document.documentElement.setAttribute("data-theme", tema);
  }

  function cambiaTono(tono: "zinc" | "slate" | "stone") {
    tonoCorrente = tono;
    document.documentElement.setAttribute("data-tone", tono);
  }

  function mostraToast() {
    toastVisibile = true;
    setTimeout(() => (toastVisibile = false), 2500);
  }
</script>

<div class="demo">
  <!-- Controlli tema/tono -->
  <header class="demo-header">
    <h1>Prompt a Porter — Componenti</h1>
    <div class="demo-controls">
      <div class="row-tight">
        <span class="eyebrow">Tema</span>
        {#each ["dark", "light"] as t}
          <Button
            dimensione="sm"
            variante={temaCorrente === t ? "primary" : "ghost"}
            onclick={() => cambiaTema(t as "dark" | "light")}
          >
            {t}
          </Button>
        {/each}
      </div>
      <div class="row-tight">
        <span class="eyebrow">Tono</span>
        {#each ["zinc", "slate", "stone"] as t}
          <Button
            dimensione="sm"
            variante={tonoCorrente === t ? "primary" : "ghost"}
            onclick={() => cambiaTono(t as "zinc" | "slate" | "stone")}
          >
            {t}
          </Button>
        {/each}
      </div>
    </div>
  </header>

  <!-- Sezioni -->
  <section class="demo-section">
    <h2 class="eyebrow">Button</h2>
    <div class="demo-row">
      <Button>Default</Button>
      <Button variante="primary">Primary</Button>
      <Button variante="private">Private</Button>
      <Button variante="ghost">Ghost</Button>
      <Button variante="danger">Danger</Button>
      <Button disabled>Disabled</Button>
    </div>
    <div class="demo-row">
      <Button dimensione="sm">Small</Button>
      <Button>Medium</Button>
      <Button dimensione="lg">Large</Button>
      <Button soloIcona>✕</Button>
    </div>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">Input / Textarea / Select</h2>
    <div class="demo-grid">
      <Field etichetta="Nome" hint="Il nome del segnaposto">
        <Input bind:valore={inputValore} placeholder="Inserisci testo…" />
      </Field>
      <Field etichetta="Modello" >
        <Select bind:valore={selectValore}>
          <option value="claude">Claude</option>
          <option value="gpt">GPT</option>
          <option value="gemini">Gemini</option>
        </Select>
      </Field>
      <Field etichetta="Campo con errore" errore="Questo campo è obbligatorio">
        <Input valore="" invalido placeholder="Obbligatorio" />
      </Field>
    </div>
    <Field etichetta="Body prompt">
      <Textarea bind:valore={textareaValore} />
    </Field>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">Switch</h2>
    <div class="demo-row">
      <span class="subtle">Team:</span>
      <Switch bind:attivo={switchAttivo} />
      <span>{switchAttivo ? "Attivo" : "Inattivo"}</span>
      <span class="subtle">Privato:</span>
      <Switch bind:attivo={switchPrivato} privato />
      <span>{switchPrivato ? "Attivo" : "Inattivo"}</span>
    </div>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">Tag / Badge / Kbd</h2>
    <div class="demo-row">
      <Tag>default</Tag>
      <Tag variante="private">privato</Tag>
      <Tag variante="team">team</Tag>
      <Tag colore="#E5A93D">con colore</Tag>
    </div>
    <div class="demo-row">
      <Badge>default</Badge>
      <Badge variante="success">success</Badge>
      <Badge variante="warning">warning</Badge>
      <Badge variante="danger">danger</Badge>
      <Badge variante="info">info</Badge>
    </div>
    <div class="demo-row">
      <Kbd>⌘</Kbd><Kbd>⇧</Kbd><Kbd>P</Kbd>
      <span class="subtle">oppure</span>
      <Kbd>Ctrl</Kbd><Kbd>Shift</Kbd><Kbd>P</Kbd>
    </div>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">Placeholder (segnaposti)</h2>
    <div class="demo-row">
      <Placeholder nome="nome" />
      <Placeholder nome="ruolo" />
      <Placeholder nome="contesto" team />
    </div>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">NavItem</h2>
    <div class="demo-nav">
      <NavItem attivo>Recenti</NavItem>
      <NavItem conteggio={12}>Preferiti</NavItem>
      <NavItem conteggio={47}>Tutti i prompt</NavItem>
    </div>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">Tooltip</h2>
    <div class="demo-row">
      <Tooltip testo="Questo è un tooltip">
        <Button dimensione="sm">Hovera qui</Button>
      </Tooltip>
      <Tooltip testo="Copia negli appunti">
        <Button variante="primary" dimensione="sm">Copia</Button>
      </Tooltip>
    </div>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">Skeleton</h2>
    <div class="col">
      <Skeleton larghezza="60%" altezza="22px" />
      <Skeleton altezza="14px" />
      <Skeleton larghezza="80%" altezza="14px" />
    </div>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">EmptyState</h2>
    <EmptyState
      titolo="Nessun prompt ancora"
      hint="Crea il tuo primo prompt per iniziare a organizzare la tua libreria."
    >
      {#snippet azioni()}
        <Button variante="primary">Crea prompt</Button>
      {/snippet}
    </EmptyState>
  </section>

  <section class="demo-section">
    <h2 class="eyebrow">Toast</h2>
    <Button onclick={mostraToast}>Mostra toast</Button>
    <Toast variante="success" visibile={toastVisibile}>
      ✓ Prompt copiato negli appunti
    </Toast>
  </section>
</div>

<style>
  .demo {
    max-width: 960px;
    margin: 0 auto;
    padding: var(--sp-6);
    display: flex;
    flex-direction: column;
    gap: var(--sp-6);
    overflow-y: auto;
    height: 100vh;
  }

  .demo-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-3);
  }

  .demo-header h1 {
    font-size: var(--fs-2xl);
    font-weight: var(--fw-bold);
    color: var(--text-strong);
  }

  .demo-controls {
    display: flex;
    gap: var(--sp-4);
  }

  .demo-section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-4);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
  }

  .demo-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    align-items: center;
  }

  .demo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--sp-3);
  }

  .demo-nav {
    width: 200px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
