<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import SceneCmdk from './scenes/SceneCmdk.vue'
import SceneLibrary from './scenes/SceneLibrary.vue'
import SceneCompile from './scenes/SceneCompile.vue'
import SceneHistory from './scenes/SceneHistory.vue'
import SceneImport from './scenes/SceneImport.vue'
import SceneWardrobe from './scenes/SceneWardrobe.vue'

// Vincoli dall'handoff: autoplay con setInterval (rAF si ferma nei tab in
// background), switch scena via display (classe .on), niente autoplay con
// prefers-reduced-motion (navigazione manuale attiva).
// Handoff UX Fix 5A: l'auto-avanzamento si ferma DEFINITIVAMENTE alla prima
// interazione (hover/focus/tocco) e riparte solo dal play esplicito;
// controlli pausa/play + contatore "n/6" + puntini di posizione.
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
const reduceMotion = ref(false)
// Stop permanente su prima interazione; il play esplicito lo revoca.
const userStopped = ref(false)
const playing = computed(() => !reduceMotion.value && !userStopped.value)
const progressPct = ref(0)
const capHtml = ref(CAPS[0])
const capVisible = ref(true)

let elapsedMs = 0
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
  if (!playing.value) return
  tickTimer = setInterval(() => {
    elapsedMs += TICK_MS
    const frac = Math.min(elapsedMs / DUR_MS[current.value], 1)
    progressPct.value = frac * 100
    if (frac >= 1) go(current.value + 1)
  }, TICK_MS)
}

function ferma(): void {
  if (userStopped.value) return
  userStopped.value = true
  clearInterval(tickTimer)
  progressPct.value = 0
}

function togglePlay(): void {
  userStopped.value = !userStopped.value
  restart()
}

// Il focusin ferma l'autoplay, MA non quando arriva dal bottone play:
// su click del mouse focusin precede click, e ferma() + togglePlay()
// si annullerebbero a vicenda (il "pausa" non pauserebbe mai).
function onFocusIn(event: FocusEvent): void {
  const target = event.target as HTMLElement | null
  if (target?.classList.contains('sc-play')) return
  ferma()
}

function onReduceMotionChange(event: MediaQueryListEvent): void {
  reduceMotion.value = event.matches
  restart()
}

// Swipe orizzontale su mobile (handoff UX Fix 1). Soglia 40px per non
// confondere lo scroll verticale con il cambio scena.
const SWIPE_MIN_PX = 40
let touchStartX = 0
let touchStartY = 0

function onTouchStart(event: TouchEvent): void {
  ferma()
  touchStartX = event.changedTouches[0].clientX
  touchStartY = event.changedTouches[0].clientY
}

function onTouchEnd(event: TouchEvent): void {
  const dx = event.changedTouches[0].clientX - touchStartX
  const dy = event.changedTouches[0].clientY - touchStartY
  if (Math.abs(dx) < SWIPE_MIN_PX || Math.abs(dx) < Math.abs(dy)) return
  go(current.value + (dx < 0 ? 1 : -1))
}

onMounted(() => {
  reduceMq = window.matchMedia('(prefers-reduced-motion: reduce)')
  reduceMotion.value = reduceMq.matches
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
  <div class="showcase" id="come-funziona" @focusin="onFocusIn">
    <div
      class="viewport"
      @mouseenter="ferma"
      @touchstart.passive="onTouchStart"
      @touchend.passive="onTouchEnd"
    >
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
      <button
        class="sc-arrow sc-play"
        type="button"
        :aria-label="playing ? 'Metti in pausa la vetrina' : 'Riavvia la vetrina'"
        :aria-pressed="!playing"
        @click="togglePlay"
      >{{ playing ? '❙❙' : '▶' }}</button>
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
      <span class="sc-count" aria-live="polite">{{ current + 1 }} / {{ SCENES.length }}</span>
      <button class="sc-arrow" type="button" aria-label="Scena successiva" @click="go(current + 1)">›</button>
    </div>
    <div class="sc-dots" role="group" aria-label="Posizione nella vetrina">
      <button
        v-for="(scene, k) in SCENES"
        :key="scene.tab"
        class="sc-dot"
        :class="{ on: k === current }"
        type="button"
        :aria-current="k === current || undefined"
        :aria-label="`Vai alla scena ${k + 1}: ${scene.tab}`"
        @click="go(k)"
      ></button>
    </div>
    <div class="sc-progress" :class="{ still: !playing }"><i :style="{ width: progressPct + '%' }"></i></div>
    <p class="sc-cap" :style="{ opacity: capVisible ? 1 : 0 }" v-html="capHtml"></p>
  </div>
</template>
