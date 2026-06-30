<script lang="ts">
  // Pagina "Informazioni" (About) — raggiungibile da Impostazioni → Guida e
  // aiuto oppure dalla voce di navigazione omonima. Mostra identità dell'app,
  // versione reale (runtime), codename del ciclo e i link canonici (sito,
  // repository, segnalazioni, licenza, changelog).
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { CODENAME } from "$lib/codename";
  import { ExternalLink } from "lucide-svelte";

  // Sorgente unica dei link al progetto. Allineata a docs-links.ts
  // (REPO_ORG_REPO) e al campo `repository` del package.
  const REPO = "https://github.com/robertomarchioro/prompt-a-porter";
  const SITO = "https://prompt-a-porter.app";

  interface VoceLink {
    etichetta: string;
    href: string;
  }

  const link: VoceLink[] = [
    { etichetta: "Sito del progetto", href: SITO },
    { etichetta: "Codice sorgente (GitHub)", href: REPO },
    { etichetta: "Segnala un problema", href: `${REPO}/issues/new/choose` },
    { etichetta: "Note di rilascio", href: `${REPO}/blob/main/CHANGELOG.md` },
    {
      etichetta: "Licenza (AGPL-3.0)",
      href: `${REPO}/blob/main/LICENSE`,
    },
  ];

  let versione = $state("");
  onMount(async () => {
    try {
      versione = await getVersion();
    } catch {
      versione = "";
    }
  });
</script>

<h3>Informazioni</h3>

<div class="identita">
  <p class="nome-app">Prompt a Porter</p>
  <p class="meta">
    {#if versione}<span class="versione">v{versione}</span>{/if}
    <span class="codename">{CODENAME}</span>
  </p>
  <p class="hint descrizione">
    Gestore di prompt desktop, locale e cifrato. Tutti i tuoi prompt restano
    sul tuo computer.
  </p>
</div>

<div class="campo">
  <span class="campo-label">Collegamenti</span>
  <ul class="about-elenco">
    {#each link as voce (voce.href)}
      <li>
        <a
          class="about-voce"
          href={voce.href}
          target="_blank"
          rel="noopener noreferrer"
          aria-label={`${voce.etichetta} (si apre nel browser)`}
        >
          <span class="about-voce-testo">{voce.etichetta}</span>
          <ExternalLink size={13} aria-hidden="true" />
        </a>
      </li>
    {/each}
  </ul>
</div>

<p class="copyright">© Roberto Marchioro — distribuito sotto licenza AGPL-3.0.</p>

<style>
  .identita {
    margin-bottom: var(--sp-3, 0.75rem);
  }

  .nome-app {
    margin: 0;
    font-size: var(--fs-lg, 1.1rem);
    font-weight: var(--fw-bold, 700);
    color: var(--text-default);
  }

  .meta {
    display: flex;
    align-items: center;
    gap: var(--sp-2, 0.5rem);
    margin: 0.25rem 0 0;
  }

  .versione {
    font-size: var(--fs-xs, 0.75rem);
    color: var(--text-subtle);
    padding: 2px var(--sp-2, 0.5rem);
    border-radius: var(--radius-full, 999px);
    border: 1px solid var(--border-subtle);
  }

  .codename {
    font-size: var(--fs-sm, 0.9rem);
    font-weight: var(--fw-medium, 500);
    color: var(--text-muted);
  }

  .descrizione {
    margin-top: 0.5rem;
  }

  .about-elenco {
    list-style: none;
    margin: 0.25rem 0 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }

  .about-voce {
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

  .about-voce:hover {
    background: var(--bg-overlay);
  }

  .about-voce:focus-visible {
    outline: 2px solid var(--accent-private);
    outline-offset: 2px;
  }

  .about-voce-testo {
    font-size: var(--fs-sm, 0.9rem);
  }

  .copyright {
    margin-top: var(--sp-3, 0.75rem);
    font-size: var(--fs-xs, 0.75rem);
    color: var(--text-subtle);
  }
</style>
