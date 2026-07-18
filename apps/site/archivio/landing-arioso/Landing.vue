<script setup lang="ts">
import TopBar from './TopBar.vue'
import HeroStage from './HeroStage.vue'
import ShowcaseCarousel from './ShowcaseCarousel.vue'
import ManifestoSection from './ManifestoSection.vue'
import ProblemSection from './ProblemSection.vue'
import CollectionGrid from './CollectionGrid.vue'
import SeasonDebut from './SeasonDebut.vue'
import SiteFooter from './SiteFooter.vue'
</script>

<template>
  <div class="pap-landing">
    <TopBar />
    <main>
      <HeroStage />
      <ManifestoSection />
      <ProblemSection />
      <!-- Handoff UX Fix 2: la vetrina completa vive qui, dopo il problema —
           non più come primo elemento auto-animato dentro l'hero. -->
      <section class="showcase-sec"><ShowcaseCarousel /></section>
      <CollectionGrid />
      <SeasonDebut />
    </main>
    <SiteFooter />
  </div>
</template>
