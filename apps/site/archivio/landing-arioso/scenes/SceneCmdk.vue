<script setup lang="ts">
import CmdkPalette, { type CmdkItem } from '../CmdkPalette.vue'

const ITEMS: CmdkItem[] = [
  { t: 'Riassumi articolo in N punti', d: 'Distilla un testo lungo in <em>N</em> punti chiave + TL;DR + verdetto.', badge: 'privato' },
  { t: 'Commit message da diff', d: 'Genera un messaggio Conventional Commits da un diff.', badge: 'privato' },
  { t: 'Email professionale parametrica', d: 'Email con tono e contenuto variabili.', badge: 'privato' },
  { t: 'Code review strutturata', d: 'Review per severità. Importa il ruolo <em>engineer</em>.', badge: 'privato' },
  { t: 'Traduzione tecnica con glossario', d: 'Traduce rispettando un glossario di termini.', badge: 'privato' },
  { t: 'Brainstorm con criteri', d: 'Genera idee valutate su criteri scelti da te.', badge: 'privato' },
]
</script>

<template>
  <CmdkPalette :items="ITEMS" />
</template>
