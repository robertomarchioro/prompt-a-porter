<script setup lang="ts">
// Landing «Scontrino cucito» — architettura §1.1 delle istruzioni:
// un solo URL, alberi separati per composizione. F1 = solo desktop (#4a);
// F2 aggiungerà LandingMobile (#3a) e la commutazione CSS a 680px.
import LandingDesktop from './desktop/LandingDesktop.vue'
</script>

<template>
  <div class="pap-landing">
    <LandingDesktop />
  </div>
</template>
