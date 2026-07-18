<script setup lang="ts">
// Composizione mobile #3a «Scontrino cucito» — ordine dei blocchi dal
// handoff docs/roadmap/website/mobile/README.md. La status bar finta del
// prototipo è omessa (solo cornice di mockup).
import TopBarMobile from './TopBarMobile.vue'
import RibbonMobile from './RibbonMobile.vue'
import HeroMobile from './HeroMobile.vue'
import PaletteMobile from './PaletteMobile.vue'
import ManifestoMobile from './ManifestoMobile.vue'
import FiloDivisore from './FiloDivisore.vue'
import ScontrinoMobile from './ScontrinoMobile.vue'
import ClientiMobile from './ClientiMobile.vue'
import SezioneDesktop from './SezioneDesktop.vue'
import FooterMobile from './FooterMobile.vue'
</script>

<template>
  <div class="pap-mobile">
    <TopBarMobile />
    <main>
      <RibbonMobile />
      <HeroMobile />
      <PaletteMobile />
      <ManifestoMobile />
      <FiloDivisore />
      <ScontrinoMobile />
      <ClientiMobile />
      <SezioneDesktop />
    </main>
    <FooterMobile />
  </div>
</template>

<style scoped>
/* Colonna singola fluida: canvas 392px, centrata fino a ~460px (handoff
   §Responsive); oltre 680px subentra l'albero desktop. */
.pap-mobile {
  max-width: 460px;
  margin: 0 auto;
}
</style>
