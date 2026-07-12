<script setup lang="ts">
import ShowcaseCarousel from './ShowcaseCarousel.vue'
import { GITHUB, RELEASES } from './links'
import { useNomeOsVisitatore } from './os'

const osNome = useNomeOsVisitatore()
</script>

<template>
  <section class="stage">
    <div class="spot"></div>
    <div class="inner">
      <div class="eyebrow"><kbd>Ctrl + Shift + P</kbd><span class="t">richiama la tua collezione, in qualsiasi app</span></div>
      <h1 class="head serif">Prompt-<em>à</em>-porter</h1>
      <p class="sub">La tua collezione di prompt AI. Versionati, su misura, <b>a un tasto di distanza</b> — ovunque tu stia scrivendo.</p>

      <div class="hero-cta">
        <a class="btn btn-primary" :href="RELEASES"><span class="ico">{{ osNome === 'Windows' ? '⊞' : '↓' }}</span> {{ osNome ? `Scarica per ${osNome}` : "Scarica l'app" }}</a>
        <a class="btn btn-ghost" :href="GITHUB">Vedi su GitHub</a>
        <a class="alt" :href="RELEASES">Altre piattaforme →</a>
      </div>
      <p class="hero-cta hero-meta"><span class="meta">gratis · local-first · v1.0</span></p>

      <ShowcaseCarousel />
    </div>
  </section>
</template>
