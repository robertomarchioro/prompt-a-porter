<script setup lang="ts">
import { ref } from 'vue'
import { withBase } from 'vitepress'
import { GITHUB, GUIDA } from '../links'

// Drawer di navigazione aperto dall'hamburger (handoff §Interactions:
// La collezione / Come funziona / Guida / GitHub).
const aperto = ref(false)

function chiudi() {
  aperto.value = false
}
</script>

<template>
  <div class="topbar">
    <div class="brand">
      <img class="mark" :src="withBase('/icons/icon-64.png')" alt="" width="26" height="26" />
      <span class="nome">Prompt a Porter</span>
    </div>
    <button
      class="hamburger"
      :aria-expanded="aperto"
      aria-label="Apri il menu di navigazione"
      @click="aperto = !aperto"
    >
      ☰
    </button>
  </div>
  <div v-if="aperto" class="drawer" @keydown.esc="chiudi">
    <nav aria-label="Navigazione principale">
      <a href="#collezione-m" @click="chiudi">La collezione</a>
      <a href="#come-funziona-m" @click="chiudi">Come funziona</a>
      <a :href="GUIDA" @click="chiudi">Guida</a>
      <a :href="GITHUB" @click="chiudi">GitHub</a>
    </nav>
  </div>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px 2px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 10px;
}
.mark {
  width: 26px;
  height: 26px;
  display: block;
}
.nome {
  font-family: var(--pap-serif);
  font-size: 17px;
}
.hamburger {
  font-size: 19px;
  color: var(--pap-faint);
  background: none;
  border: none;
  cursor: pointer;
  /* hit target ≥44px (§2.4 istruzioni) */
  min-width: 44px;
  min-height: 44px;
  margin: -10px -12px -10px 0;
}
.drawer {
  border-bottom: 1px solid var(--pap-line);
  background: var(--pap-surface);
}
.drawer nav {
  display: flex;
  flex-direction: column;
  padding: 6px 0;
}
.drawer a {
  font-size: 15px;
  color: var(--pap-sub);
  padding: 13px 24px;
  min-height: 44px;
  display: flex;
  align-items: center;
}
.drawer a:hover {
  color: var(--pap-viola);
}
</style>
