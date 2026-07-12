<script setup lang="ts">
import { ref } from 'vue'
import { withBase } from 'vitepress'
import { GITHUB } from './links'
import { useLinkDownload } from './download'

const linkDownload = useLinkDownload()

// Handoff UX Fix 5B: sotto i 720px la nav non sparisce più — un hamburger
// apre le stesse voci; si chiude su selezione voce e su Esc, restituendo
// il focus al bottone (altrimenti resterebbe su un link display:none).
const menuAperto = ref(false)
const hamEl = ref<HTMLButtonElement | null>(null)

function chiudiMenu(): void {
  menuAperto.value = false
}

// Su Esc il focus torna al bottone; su selezione voce invece segue l'àncora.
function chiudiConFocus(): void {
  if (!menuAperto.value) return
  chiudiMenu()
  hamEl.value?.focus()
}
</script>

<template>
  <header class="topbar" @keydown.escape="chiudiConFocus">
    <div class="brand"><img class="mk" :src="withBase('/icons/icon-64.png')" alt="" width="30" height="30" /><span class="nm">Prompt&nbsp;a&nbsp;Porter</span></div>
    <button
      ref="hamEl"
      class="ham"
      type="button"
      aria-controls="menu-principale"
      :aria-expanded="menuAperto"
      :aria-label="menuAperto ? 'Chiudi il menu' : 'Apri il menu'"
      @click="menuAperto = !menuAperto"
    ><i></i><i></i><i></i></button>
    <nav id="menu-principale" class="nav" :class="{ aperta: menuAperto }">
      <a href="#collezione" @click="chiudiMenu">La collezione</a>
      <a href="#come-funziona" @click="chiudiMenu">Come funziona</a>
      <a :href="GITHUB" @click="chiudiMenu">GitHub</a>
      <a class="dl" :href="linkDownload" @click="chiudiMenu">Scarica</a>
    </nav>
  </header>
</template>
