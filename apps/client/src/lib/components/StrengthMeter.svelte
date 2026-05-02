<script lang="ts">
  interface Props {
    password: string;
  }

  let { password }: Props = $props();

  function calcolaEntropia(pwd: string): number {
    if (pwd.length === 0) return 0;
    let charset = 0;
    if (/[a-z]/.test(pwd)) charset += 26;
    if (/[A-Z]/.test(pwd)) charset += 26;
    if (/[0-9]/.test(pwd)) charset += 10;
    if (/[^a-zA-Z0-9]/.test(pwd)) charset += 32;
    return Math.floor(pwd.length * Math.log2(charset || 1));
  }

  const entropia = $derived(calcolaEntropia(password));

  const livello = $derived.by(() => {
    if (entropia < 28) return 1;
    if (entropia < 36) return 2;
    if (entropia < 60) return 3;
    return 4;
  });

  const etichetta = $derived.by(() => {
    switch (livello) {
      case 1:
        return "Debole";
      case 2:
        return "Sufficiente";
      case 3:
        return "Buona";
      default:
        return "Forte";
    }
  });
</script>

<div class="meter">
  <div class="meter-bars">
    {#each [1, 2, 3, 4] as i}
      <div
        class="meter-bar"
        class:attivo={i <= livello && password.length > 0}
        data-livello={livello}
      ></div>
    {/each}
  </div>
  {#if password.length > 0}
    <span class="meter-label">
      Forza: {etichetta} · ~{entropia} bit di entropia
    </span>
  {/if}
</div>

<style>
  .meter {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .meter-bars {
    display: flex;
    gap: 3px;
  }

  .meter-bar {
    height: 4px;
    flex: 1;
    background: var(--border-subtle);
    border-radius: var(--radius-full);
    transition: background var(--motion-fast);
  }

  .meter-bar.attivo[data-livello="1"] {
    background: var(--danger);
  }

  .meter-bar.attivo[data-livello="2"] {
    background: var(--warning);
  }

  .meter-bar.attivo[data-livello="3"],
  .meter-bar.attivo[data-livello="4"] {
    background: var(--success);
  }

  .meter-label {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }
</style>
