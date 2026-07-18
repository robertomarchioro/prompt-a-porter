<script setup lang="ts">
// Mockup della command palette (Ctrl+Shift+P). Il campo `d` può contenere
// markup <em> statico definito nei componenti chiamanti — mai input utente.
export interface CmdkItem {
  t: string
  d: string
  badge?: string
}

withDefaults(
  defineProps<{
    items: CmdkItem[]
    showEnterHint?: boolean
  }>(),
  { showEnterHint: true },
)
</script>

<template>
  <div class="cmdk">
    <div class="cmdk-search">
      <svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="7" /><line x1="16.5" y1="16.5" x2="21" y2="21" /></svg>
      <span class="ph">Cerca prompt, tag o azione…</span><span class="caret"></span><span class="esc">esc</span>
    </div>
    <div class="cmdk-lab">Recenti</div>
    <div v-for="(item, k) in items" :key="item.t" class="cmdk-item" :class="{ active: k === 0 }">
      <div class="b">
        <div class="t" v-text="item.t"></div>
        <div class="d" v-html="item.d"></div>
      </div>
      <span v-if="item.badge" class="bd" v-text="item.badge"></span>
    </div>
    <div class="cmdk-foot">
      <span><kbd>esc</kbd> chiudi</span>
      <span><kbd>↑</kbd><kbd>↓</kbd> naviga</span>
      <span v-if="showEnterHint"><kbd>↵</kbd> seleziona</span>
      <span class="push"><kbd>⌃</kbd><kbd>↵</kbd> compila e incolla</span>
    </div>
  </div>
</template>
