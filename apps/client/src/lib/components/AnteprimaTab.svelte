<script lang="ts">
  /**
   * Tab Anteprima del DetailPane (F5 PR-A).
   *
   * Render del body con segnaposti `{{nome}}` e import `{{import "x"}}`
   * evidenziati visivamente. NON risolve i default né esegue gli import:
   * F5 PR-A.x potrà aggiungere quando F8 modale Compila introdurrà
   * storage di default values.
   *
   * Riferimento blueprint: docs/roadmap/redesign-v08/blueprint-F5.md §1
   */

  type Segmento =
    | { tipo: "testo"; testo: string }
    | { tipo: "segnaposto"; nome: string }
    | { tipo: "import"; path: string };

  const RE = /(\{\{\s*import\s+"([^"]+)"\s*\}\})|(\{\{\s*(\w+)\s*\}\})/g;

  function parsa(testo: string): Segmento[] {
    const acc: Segmento[] = [];
    let last = 0;
    let m: RegExpExecArray | null;
    RE.lastIndex = 0;
    while ((m = RE.exec(testo)) !== null) {
      if (m.index > last) {
        acc.push({ tipo: "testo", testo: testo.slice(last, m.index) });
      }
      if (m[2] !== undefined) {
        acc.push({ tipo: "import", path: m[2] });
      } else if (m[4] !== undefined) {
        acc.push({ tipo: "segnaposto", nome: m[4] });
      }
      last = m.index + m[0].length;
    }
    if (last < testo.length) {
      acc.push({ tipo: "testo", testo: testo.slice(last) });
    }
    return acc;
  }

  interface Props {
    body: string;
  }

  let { body }: Props = $props();

  const segmenti = $derived(parsa(body));
</script>

{#if body.trim().length === 0}
  <div class="vuoto">
    <p>Body vuoto. Aggiungi contenuto nel tab Editor.</p>
  </div>
{:else}
  <pre class="anteprima">{#each segmenti as seg, i (i)}{#if seg.tipo === "testo"}{seg.testo}{:else if seg.tipo === "segnaposto"}<span
          class="ph"
          title="Segnaposto: sostituito al momento della compilazione (F8)"
          >{`{{${seg.nome}}}`}</span>{:else}<span
          class="imp"
          title="Import composto: risolto al momento della compilazione (F8)"
          >{`{{import "${seg.path}"}}`}</span>{/if}{/each}</pre>
{/if}

<style>
  .anteprima {
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.65;
    background: var(--bg-surface);
    color: var(--text-default);
    padding: var(--sp-3);
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    height: 100%;
    overflow-y: auto;
  }

  .ph {
    background: var(--accent-private-soft);
    color: var(--accent-private);
    border-radius: var(--radius-sm);
    padding: 1px 4px;
    font-weight: var(--fw-medium);
  }

  .imp {
    background: var(--info-soft);
    color: var(--info);
    border-radius: var(--radius-sm);
    padding: 1px 4px;
    text-decoration: underline dotted;
    font-weight: var(--fw-medium);
  }

  .vuoto {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .vuoto p {
    margin: 0;
  }
</style>
