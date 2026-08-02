<script lang="ts">
  /**
   * Diff viewer side-by-side / unified (V0.8 F5 PR-D).
   *
   * Genera un unified diff con jsdiff `createTwoFilesPatch`, poi delega
   * a diff2html per il render. Toggle persistito in localStorage per-utente
   * (decisione designer #9). Fallback automatico unified sotto 900px viewport.
   *
   * Riferimenti:
   * - Decisione designer #9
   * - Blueprint F5 PR-D §4
   */
  import { createTwoFilesPatch } from "diff";
  import { html as diff2htmlHtml } from "diff2html";
  import "diff2html/bundles/css/diff2html.min.css";
  import { Columns, AlignLeft } from "lucide-svelte";

  type Modalita = "side-by-side" | "line-by-line";

  const STORAGE_KEY = "pap.diff.modalita";
  const RESPONSIVE_PX = 900;

  function caricaModalita(): Modalita {
    try {
      const v = localStorage.getItem(STORAGE_KEY);
      return v === "line-by-line" || v === "side-by-side"
        ? v
        : "side-by-side";
    } catch {
      return "side-by-side";
    }
  }
  function salvaModalita(m: Modalita): void {
    try {
      localStorage.setItem(STORAGE_KEY, m);
    } catch {
      /* ignore */
    }
  }

  /**
   * "contenitore" (default): il DiffViewer riempie un antenato ad altezza
   * DEFINITA (height:100%) e scrolla al proprio interno. Comportamento
   * storico, usato da CronologiaTab dentro un pannello ad altezza fissa —
   * NON toccare per non farlo regredire.
   *
   * "contenuto": il DiffViewer si dimensiona sulla propria altezza
   * intrinseca (senza flex:1/flex-basis:0) e delega lo scroll a un
   * antenato del consumer. Serve a RitoccoModal, il cui wrapper ha solo
   * `max-height` (vedi #506/#514/#553 — i due tentativi precedenti
   * patchavano il consumer con `:global()`, qui il prop lo rende un caso
   * di prima classe del componente).
   */
  type Altezza = "contenitore" | "contenuto";

  interface Props {
    bodyA: string;
    bodyB: string;
    etichettaA?: string;
    etichettaB?: string;
    altezza?: Altezza;
  }

  let {
    bodyA,
    bodyB,
    etichettaA = "Versione A",
    etichettaB = "Versione B",
    altezza = "contenitore",
  }: Props = $props();

  let preferenzaUtente = $state<Modalita>(caricaModalita());
  let larghezza = $state(
    typeof window !== "undefined" ? window.innerWidth : 1200,
  );

  function onResize(): void {
    larghezza = window.innerWidth;
  }

  $effect(() => {
    if (typeof window === "undefined") return;
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  function setModalita(m: Modalita): void {
    preferenzaUtente = m;
    salvaModalita(m);
  }

  // Modalità effettiva: forza unified sotto 900px (decisione #9 responsive)
  const modalitaEffettiva = $derived<Modalita>(
    larghezza < RESPONSIVE_PX ? "line-by-line" : preferenzaUtente,
  );

  const renderHtml = $derived.by(() => {
    if (bodyA === bodyB) return "";
    const patch = createTwoFilesPatch(
      etichettaA,
      etichettaB,
      bodyA,
      bodyB,
      "",
      "",
      { context: 3 },
    );
    return diff2htmlHtml(patch, {
      drawFileList: false,
      outputFormat: modalitaEffettiva,
      matching: "lines",
      diffStyle: "word",
    });
  });
</script>

<div class="diff-viewer" data-altezza={altezza}>
  <header class="header">
    <span class="lbl">Diff</span>
    <div class="toggle" role="group" aria-label="Modalità diff">
      <button
        type="button"
        class="t-btn"
        data-attivo={preferenzaUtente === "side-by-side" || undefined}
        disabled={larghezza < RESPONSIVE_PX}
        onclick={() => setModalita("side-by-side")}
        title={larghezza < RESPONSIVE_PX
          ? "Side-by-side disabilitato sotto 900px"
          : "Side-by-side"}
      >
        <Columns size={12} />
        <span>Side-by-side</span>
      </button>
      <button
        type="button"
        class="t-btn"
        data-attivo={preferenzaUtente === "line-by-line" || undefined}
        onclick={() => setModalita("line-by-line")}
        title="Unified"
      >
        <AlignLeft size={12} />
        <span>Unified</span>
      </button>
    </div>
  </header>

  <div class="render">
    {#if bodyA === bodyB}
      <p class="vuoto">Nessuna differenza tra le due versioni.</p>
    {:else}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html renderHtml}
    {/if}
  </div>
</div>

<style>
  .diff-viewer {
    display: flex;
    flex-direction: column;
    background: var(--bg-canvas);
  }

  /* Modalità "contenitore": altezza definita dal genitore, scroll interno.
     Comportamento storico (v. CronologiaTab). */
  .diff-viewer[data-altezza="contenitore"] {
    height: 100%;
    overflow: hidden;
  }

  /* Modalità "contenuto": altezza intrinseca, nessuno scroll qui — lo
     scroll è responsabilità di un antenato del consumer (v. RitoccoModal
     `.diff-scroll`). Niente flex:1/flex-basis:0 sui discendenti: quella
     combinazione dentro un flex column ad altezza `auto` è il caso limite
     che #514 non aveva risolto (Chromium e WebView2 lo risolvono
     diversamente — vedi nota nel `.render` sotto). */
  .diff-viewer[data-altezza="contenuto"] {
    height: auto;
    overflow: visible;
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
  }

  .lbl {
    font-size: 10px;
    color: var(--text-subtle);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    font-weight: var(--fw-semibold);
  }

  .toggle {
    margin-left: auto;
    display: inline-flex;
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    padding: 2px;
  }

  .t-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    padding: 2px 8px;
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
    font-family: var(--font-ui);
  }

  .t-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .t-btn[data-attivo] {
    background: var(--bg-surface);
    color: var(--text-default);
  }

  /* line-height e font-size sono dichiarati qui esplicitamente (non
     lasciati all'eredità) perché si propagano sull'HTML iniettato via
     `{@html}`: diff2html non fissa un proprio line-height e posiziona i
     numeri di riga con `position:absolute` senza top/left, assumendo
     un'altezza di riga stabile e coerente in tutto l'albero — un valore
     ambiguo o ereditato da un antenato del consumer produce lo scenario
     "le righe si sovrappongono" di #553. */
  .render {
    overflow: auto;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
    /* #574 — causa reale delle recidive #506/#514/#553: diff2html
       posiziona `.d2h-code-linenumber`/`.d2h-code-side-linenumber` con
       `position:absolute` ma NON dichiara `position:relative` su nessun
       proprio antenato (`.d2h-file-wrapper`, `.d2h-file-diff`,
       `.d2h-code-line` sono tutti `static`). Senza un containing block
       nell'albero, l'unico antenato posizionato risultava il backdrop
       `fixed` della modale (v. Modale.svelte) — il gutter restava
       ancorato lì invece che scorrere col contenuto, causando
       l'overlap. `overflow:auto` da solo NON crea un containing block
       per un discendente `absolute`: serve `position:relative` qui, sul
       nodo che contiene l'HTML iniettato via `{@html}` — in entrambe le
       modalità `data-altezza` questo nodo è sempre dentro l'antenato
       che scrolla davvero (se stesso in "contenitore", `.diff-scroll`
       del consumer in "contenuto"), mai fuori da esso. */
    position: relative;
  }

  .diff-viewer[data-altezza="contenitore"] .render {
    /* min-height:0 evita che il contenuto forzi l'altezza del flex item
       oltre il contenitore ad altezza definita (classico bug flex+overflow). */
    flex: 1;
    min-height: 0;
  }

  .diff-viewer[data-altezza="contenuto"] .render {
    /* Nessun flex:1 qui: in un antenato ad altezza `auto` (data-altezza
       "contenuto"), flex-basis:0 + flex-grow:1 crea una dipendenza
       circolare fra l'altezza del contenitore (auto, dal contenuto) e
       quella dell'item (dal container) — Chromium e WebView2 la
       risolvono diversamente. `flex: none` fa sì che .render segua
       semplicemente la propria altezza intrinseca. */
    flex: none;
    overflow: visible;
  }

  .vuoto {
    color: var(--text-subtle);
    text-align: center;
    padding: var(--sp-4);
    margin: 0;
  }

  :global(.diff-viewer .d2h-wrapper) {
    background: var(--bg-canvas);
  }

  :global(.diff-viewer .d2h-file-header) {
    display: none;
  }

  :global(.diff-viewer .d2h-code-line),
  :global(.diff-viewer .d2h-code-side-line) {
    font-family: var(--font-mono);
  }

  :global(.diff-viewer .d2h-ins) {
    background: var(--success-soft);
    color: var(--success);
  }

  :global(.diff-viewer .d2h-del) {
    background: var(--danger-soft);
    color: var(--danger);
  }

  :global(.diff-viewer .d2h-info) {
    background: var(--bg-overlay);
    color: var(--text-subtle);
  }
</style>
