<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import SceneCmdk from './scenes/SceneCmdk.vue'
import SceneLibrary from './scenes/SceneLibrary.vue'
import SceneCompile from './scenes/SceneCompile.vue'
import SceneHistory from './scenes/SceneHistory.vue'
import SceneImport from './scenes/SceneImport.vue'
import SceneWardrobe from './scenes/SceneWardrobe.vue'

// Vincoli dall'handoff: autoplay con setInterval (rAF si ferma nei tab in
// background), switch scena via display (classe .on), pausa su hover,
// niente autoplay con prefers-reduced-motion (navigazione manuale attiva).
const SCENES = [
  { component: SceneCmdk, tab: 'La palette' },
  { component: SceneLibrary, tab: 'La libreria' },
  { component: SceneCompile, tab: 'Compila e incolla' },
  { component: SceneHistory, tab: 'Cronologia' },
  { component: SceneImport, tab: 'Import' },
  { component: SceneWardrobe, tab: 'Giorno · Sera' },
]

// Caption per scena: HTML statico definito qui, mai input utente.
const CAPS = [
  'Premi <b>⌃⇧P</b> ovunque e richiama la tua collezione, senza cambiare finestra.',
  'Tutti i tuoi prompt in un posto solo. Versioni, segnaposti, tag — <b>tutto in locale</b>.',
  'Riempi i <b>segnaposti</b> e il prompt compilato finisce dritto nel campo di testo.',
  'Ogni salvataggio è uno snapshot. <b>Diff git-style</b> e ritorno a qualsiasi versione.',
  'Cuci insieme i tuoi prompt: <b>importa ruoli e stili</b> e riusali in tutta la collezione.',
  'Tema chiaro o scuro — <b>abito da giorno o da sera</b>, la collezione è sempre la stessa.',
]

const DUR_MS = [4200, 5400, 5400, 5400, 5400, 5400]
const TICK_MS = 100
const CAP_FADE_MS = 200

const current = ref(0)
// Pausa autoplay sia su hover (mouse) sia su focus (tastiera — WCAG 2.2.2).
const hovered = ref(false)
const focused = ref(false)
const paused = computed(() => hovered.value || focused.value)
const progressPct = ref(0)
const capHtml = ref(CAPS[0])
const capVisible = ref(true)

let elapsedMs = 0
let reduceMotion = false
let reduceMq: MediaQueryList | undefined
let tickTimer: ReturnType<typeof setInterval> | undefined
let capTimer: ReturnType<typeof setTimeout> | undefined

function paintCaption(n: number): void {
  capVisible.value = false
  clearTimeout(capTimer)
  capTimer = setTimeout(() => {
    capHtml.value = CAPS[n]
    capVisible.value = true
  }, CAP_FADE_MS)
}

function go(n: number): void {
  current.value = (n + SCENES.length) % SCENES.length
  paintCaption(current.value)
  restart()
}

function restart(): void {
  clearInterval(tickTimer)
  elapsedMs = 0
  progressPct.value = 0
  if (reduceMotion) return
  tickTimer = setInterval(() => {
    if (paused.value) return
    elapsedMs += TICK_MS
    const frac = Math.min(elapsedMs / DUR_MS[current.value], 1)
    progressPct.value = frac * 100
    if (frac >= 1) go(current.value + 1)
  }, TICK_MS)
}

function onReduceMotionChange(event: MediaQueryListEvent): void {
  reduceMotion = event.matches
  restart()
}

onMounted(() => {
  reduceMq = window.matchMedia('(prefers-reduced-motion: reduce)')
  reduceMotion = reduceMq.matches
  reduceMq.addEventListener('change', onReduceMotionChange)
  restart()
})

onBeforeUnmount(() => {
  clearInterval(tickTimer)
  clearTimeout(capTimer)
  reduceMq?.removeEventListener('change', onReduceMotionChange)
})
</script>

<template>
  <div class="showcase" id="come-funziona" @focusin="focused = true" @focusout="focused = false">
    <div class="viewport" @mouseenter="hovered = true" @mouseleave="hovered = false">
      <div
        v-for="(scene, k) in SCENES"
        :key="scene.tab"
        class="scene"
        :class="{ on: k === current }"
      >
        <component :is="scene.component" />
      </div>
    </div>

    <div class="sc-controls">
      <button class="sc-arrow" type="button" aria-label="Scena precedente" @click="go(current - 1)">‹</button>
      <div class="tabline">
        <button
          v-for="(scene, k) in SCENES"
          :key="scene.tab"
          class="tab"
          :class="{ on: k === current }"
          type="button"
          @click="go(k)"
          v-text="scene.tab"
        ></button>
      </div>
      <button class="sc-arrow" type="button" aria-label="Scena successiva" @click="go(current + 1)">›</button>
    </div>
    <div class="sc-progress"><i :style="{ width: progressPct + '%' }"></i></div>
    <p class="sc-cap" :style="{ opacity: capVisible ? 1 : 0 }" v-html="capHtml"></p>
  </div>
</template>
