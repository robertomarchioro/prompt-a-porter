<script lang="ts">
  interface Props {
    nome: string;
    team?: boolean;
  }

  let { nome, team = false }: Props = $props();
</script>

<span class="ph" class:ph--team={team}>
  <span class="br">{"{{"}}</span>{nome}<span class="br">{"}}"}</span>
</span>

<style>
  .ph {
    display: inline;
    font-family: var(--font-mono);
    color: var(--accent-private);
    background: var(--accent-private-soft);
    border-radius: var(--radius-sm);
    padding: 1px 6px;
    font-weight: var(--fw-medium);
    white-space: nowrap;
  }

  .ph .br {
    opacity: 0.55;
    font-weight: var(--fw-regular);
  }

  .ph--team {
    color: var(--accent-team);
    background: var(--accent-team-soft);
  }
</style>
