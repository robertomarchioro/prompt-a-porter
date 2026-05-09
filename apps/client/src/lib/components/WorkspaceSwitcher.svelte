<script lang="ts">
  import { ChevronDown } from "lucide-svelte";
  import { coloreAvatar } from "$lib/util/avatar-color";

  interface Props {
    nome: string;
  }

  let { nome }: Props = $props();

  const colori = $derived(coloreAvatar(nome));
  const iniziale = $derived(nome.charAt(0).toUpperCase());
</script>

<!--
  Placeholder visivo non interattivo (decisione designer #2): oggi 1 vault/utente,
  multi-vault rinviato a v0.9. Tooltip native via attributo title.
-->
<div
  class="switcher"
  title="Multi-vault in arrivo — v0.9"
  aria-label="Workspace corrente: {nome}. Multi-vault disponibile in v0.9"
>
  <span
    class="avatar"
    style:background={colori.background}
    style:color={colori.foreground}
  >
    {iniziale}
  </span>
  <span class="nome">{nome}</span>
  <span class="chevron-wrap" aria-hidden="true">
    <ChevronDown size={14} />
  </span>
</div>

<style>
  .switcher {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    cursor: default;
    flex: 1;
  }

  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--radius-full);
    font-weight: var(--fw-semibold);
    font-size: var(--fs-xs);
  }

  .nome {
    flex: 1;
    color: var(--text-default);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
  }

  .chevron-wrap {
    display: inline-flex;
    color: var(--text-subtle);
    opacity: 0.4;
  }
</style>
