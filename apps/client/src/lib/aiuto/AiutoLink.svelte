<script lang="ts">
  // Guida interattiva — Fase 0: piccola affordance "?" che apre la pagina di
  // documentazione corrispondente nel browser di sistema. Accessibile per
  // natura (è un <a> focalizzabile, con aria-label esplicito).
  import { urlDoc, titoloDoc, type ChiaveDoc } from "./docs-links";

  interface Props {
    /** Quale pagina di documentazione aprire. */
    chiave: ChiaveDoc;
    /** Override dell'etichetta accessibile (default: titolo della pagina). */
    etichetta?: string;
    /** Dimensione del badge in px (default 18). */
    dimensione?: number;
  }

  let { chiave, etichetta, dimensione = 18 }: Props = $props();

  // Minimo 18px: sotto la soglia il target diventa troppo piccolo per la
  // navigazione da tastiera/tocco (WCAG 2.5.5).
  const dim = $derived(Math.max(18, dimensione));
  const titolo = $derived(etichetta ?? titoloDoc(chiave));
  const aria = $derived(`Apri la guida: ${titolo} (si apre nel browser)`);
</script>

<a
  class="aiuto-link"
  href={urlDoc(chiave)}
  target="_blank"
  rel="noopener noreferrer"
  aria-label={aria}
  title={`${titolo} — apri la guida`}
  style:width={`${dim}px`}
  style:height={`${dim}px`}
>
  <span aria-hidden="true">?</span>
</a>

<style>
  .aiuto-link {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    border: 1px solid var(--border-default);
    background: transparent;
    color: var(--text-muted);
    font-size: 0.72rem;
    font-weight: 700;
    line-height: 1;
    text-decoration: none;
    cursor: pointer;
    flex: none;
    transition:
      color 0.12s ease,
      border-color 0.12s ease,
      background 0.12s ease;
  }

  .aiuto-link:hover,
  .aiuto-link:focus-visible {
    color: var(--accent-private);
    border-color: var(--accent-private);
    background: var(--accent-private-soft);
  }

  .aiuto-link:focus-visible {
    outline: 2px solid var(--accent-private);
    outline-offset: 2px;
  }
</style>
