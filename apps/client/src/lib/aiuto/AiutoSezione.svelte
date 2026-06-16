<script lang="ts">
  // Guida interattiva — Fase 0: hub "Guida e aiuto" (sezione di Impostazioni).
  // È lo strato 3 del blueprint (menu Guida): raccolta dei link alla
  // documentazione utente, raggruppati per tema. La profondità sta online; qui
  // solo le porte d'ingresso. Il tour guidato (Fase 1) si aggancerà qui.
  import { urlDoc, titoloDoc, type ChiaveDoc } from "./docs-links";

  interface Gruppo {
    titolo: string;
    voci: ChiaveDoc[];
  }

  const gruppi: Gruppo[] = [
    {
      titolo: "Iniziare",
      voci: ["getting-started", "scorciatoie", "ricerca-semantica"],
    },
    {
      titolo: "Scrivere i prompt",
      voci: ["glossario-sintassi", "segnaposti-globali", "prompt-componibili"],
    },
    {
      titolo: "Qualità e workflow",
      voci: ["varianti", "rating", "regression-testing", "linting"],
    },
    {
      titolo: "Organizzare e condividere",
      voci: ["cartelle", "fork", "markdown-import-export", "export-json"],
    },
    {
      titolo: "Sistema e integrazioni",
      voci: ["auto-update", "cli", "mcp", "troubleshooting"],
    },
  ];
</script>

<h3>Guida e aiuto</h3>
<p class="hint">
  Spiegazioni brevi qui dentro, approfondimenti nella documentazione online (si
  apre nel browser). Presto arriverà anche un tour guidato dell'interfaccia.
</p>

{#each gruppi as gruppo (gruppo.titolo)}
  <div class="campo">
    <span class="campo-label">{gruppo.titolo}</span>
    <ul class="aiuto-elenco">
      {#each gruppo.voci as chiave (chiave)}
        <li>
          <a
            class="aiuto-voce"
            href={urlDoc(chiave)}
            target="_blank"
            rel="noopener noreferrer"
            aria-label={`${titoloDoc(chiave)} (si apre nel browser)`}
          >
            <span class="aiuto-voce-testo">{titoloDoc(chiave)}</span>
            <span class="aiuto-voce-ext" aria-hidden="true">↗</span>
          </a>
        </li>
      {/each}
    </ul>
  </div>
{/each}

<style>
  .aiuto-elenco {
    list-style: none;
    margin: 0.25rem 0 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }

  .aiuto-voce {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.4rem 0.55rem;
    border-radius: 6px;
    color: var(--text-default);
    text-decoration: none;
    transition: background 0.12s ease;
  }

  .aiuto-voce:hover,
  .aiuto-voce:focus-visible {
    background: var(--bg-raised);
    color: var(--accent-private);
  }

  .aiuto-voce:focus-visible {
    outline: 2px solid var(--accent-private);
    outline-offset: -2px;
  }

  .aiuto-voce-ext {
    color: var(--text-subtle);
    font-size: 0.85em;
    flex: none;
  }
</style>
