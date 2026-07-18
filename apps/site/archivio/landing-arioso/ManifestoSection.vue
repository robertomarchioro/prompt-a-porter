<script setup lang="ts">
import CmdkPalette, { type CmdkItem } from './CmdkPalette.vue'

const ITEMS: CmdkItem[] = [
  { t: 'Riassumi articolo in N punti', d: 'Distilla un testo lungo in <em>N</em> punti + TL;DR.', badge: 'privato' },
  { t: 'Commit message da diff', d: 'Conventional Commits da un diff.', badge: 'privato' },
  { t: 'Email professionale parametrica', d: 'Tono e contenuto variabili.', badge: 'privato' },
]
</script>

<template>
  <section class="manifesto">
    <div class="in">
      <div class="m-copy">
        <h2 class="slogan">Se ci puoi scrivere,<br />ci puoi <em>incollare il tuo prompt</em>.</h2>
        <p class="tech">Si abbina con qualsiasi campo di testo, in ogni editor, applicazione o sito web.</p>
      </div>
      <div class="m-art">
        <!-- Palette in versione light (tema locale via CSS .manifesto .m-art .cmdk*) -->
        <CmdkPalette :items="ITEMS" :show-enter-hint="false" />
      </div>
    </div>
  </section>
</template>
