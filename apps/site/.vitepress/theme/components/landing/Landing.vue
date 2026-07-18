<script setup lang="ts">
// Landing «Scontrino cucito» — architettura §1.1 delle istruzioni:
// un solo URL, due alberi di componenti separati. La commutazione a
// 680px è in CSS puro (landing.css): niente JS di detection, niente
// flash — entrambi gli alberi sono nel DOM, uno solo è visibile.
import LandingDesktop from './desktop/LandingDesktop.vue'
import LandingMobile from './mobile/LandingMobile.vue'
</script>

<template>
  <div class="pap-landing">
    <LandingDesktop class="pap-albero-desktop" />
    <LandingMobile class="pap-albero-mobile" />
  </div>
</template>
