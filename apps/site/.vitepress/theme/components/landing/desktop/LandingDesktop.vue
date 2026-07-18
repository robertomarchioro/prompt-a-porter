<script setup lang="ts">
// Composizione desktop #4a — ordine dei blocchi dal handoff
// docs/roadmap/website/desktop/README.md.
import RibbonLancio from './RibbonLancio.vue'
import TopBarDesktop from './TopBarDesktop.vue'
import HeroDesktop from './HeroDesktop.vue'
import PaletteShowcase from './PaletteShowcase.vue'
import ManifestoDesktop from './ManifestoDesktop.vue'
import CollezioneServizi from './CollezioneServizi.vue'
import ClientiTipo from './ClientiTipo.vue'
import BandaDownload from './BandaDownload.vue'
import FooterDesktop from './FooterDesktop.vue'
</script>

<template>
  <div class="pap-desktop">
    <RibbonLancio />
    <TopBarDesktop />
    <main>
      <HeroDesktop />
      <PaletteShowcase />
      <ManifestoDesktop />
      <CollezioneServizi />
      <ClientiTipo />
      <BandaDownload />
    </main>
    <FooterDesktop />
  </div>
</template>
