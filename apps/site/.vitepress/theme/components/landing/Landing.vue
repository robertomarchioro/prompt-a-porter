<script setup lang="ts">
import LaunchRibbon from './LaunchRibbon.vue'
import TopBar from './TopBar.vue'
import HeroStage from './HeroStage.vue'
import ManifestoSection from './ManifestoSection.vue'
import ProblemSection from './ProblemSection.vue'
import CollectionGrid from './CollectionGrid.vue'
import SeasonDebut from './SeasonDebut.vue'
import SiteFooter from './SiteFooter.vue'
</script>

<template>
  <div class="pap-landing">
    <LaunchRibbon />
    <TopBar />
    <main>
      <HeroStage />
      <ManifestoSection />
      <ProblemSection />
      <CollectionGrid />
      <SeasonDebut />
    </main>
    <SiteFooter />
  </div>
</template>
