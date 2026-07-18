<script setup lang="ts">
import { computed } from 'vue'
import CmdkPalette, { type CmdkItem } from './CmdkPalette.vue'
import { GITHUB, RELEASES } from './links'
import { useNomeOsVisitatore } from './os'
import { useLinkDownload } from './download'

const osNome = useNomeOsVisitatore()
const linkDownload = useLinkDownload()

// Handoff UX Fix 3: accanto alla CTA compare il formato del file per OS.
const FORMATO: Record<string, string> = {
  Windows: 'installer x64',
  macOS: '.dmg universale',
  Linux: '.AppImage x64',
}
const TUTTE = ['Windows', 'macOS', 'Linux']
const altrePiattaforme = computed(() => TUTTE.filter((n) => n !== osNome.value))

// Handoff UX Fix 2: nell'hero un solo artefatto — la command palette —
// al posto della vetrina completa auto-rotante (che scorre più in basso).
const ITEMS: CmdkItem[] = [
  { t: 'Riassumi articolo in N punti', d: 'Distilla un testo lungo in <em>N</em> punti chiave + TL;DR.', badge: 'privato' },
  { t: 'Commit message da diff', d: 'Genera un messaggio Conventional Commits da un diff.', badge: 'privato' },
  { t: 'Email professionale parametrica', d: 'Email con tono e contenuto variabili.', badge: 'privato' },
]
</script>

<template>
  <section class="stage">
    <div class="spot"></div>
    <div class="inner">
      <div class="eyebrow"><kbd>Ctrl + Shift + P</kbd><span class="t">richiama la tua collezione, in qualsiasi app</span></div>
      <h1 class="head serif">Prompt-<em>à</em>-porter</h1>
      <p class="sub">I tuoi prompt migliori, fuori dal cassetto. Libreria locale di prompt AI: cerca, compila i campi, <b>incolla dove stai scrivendo</b>.</p>

      <div class="hero-cta">
        <a class="btn btn-primary" :href="linkDownload"><span class="ico">{{ osNome === 'Windows' ? '⊞' : '↓' }}</span> {{ osNome ? `Scarica per ${osNome}` : "Scarica l'app" }}<span v-if="osNome" class="fmt">· {{ FORMATO[osNome] }}</span></a>
      </div>
      <p class="hero-alt">
        <template v-for="(nome, i) in altrePiattaforme" :key="nome">
          <a :href="RELEASES">{{ nome }}</a><span v-if="i < altrePiattaforme.length - 1"> / </span>
        </template>
        <span class="sep">·</span>
        <a :href="GITHUB">Vedi su GitHub</a>
      </p>
      <p class="hero-cta hero-meta"><span class="meta">gratis · local-first · v1.0</span></p>

      <div class="hero-palette"><CmdkPalette :items="ITEMS" /></div>
    </div>
  </section>
</template>
