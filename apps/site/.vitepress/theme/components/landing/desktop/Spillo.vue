<script setup lang="ts">
// Spillo da sarto (SVG) — testa colorata + ago, con ombra portata.
// `verso`: sx = inclinato a sinistra, dx = specchiato, giu = verticale.
withDefaults(
  defineProps<{ testa?: string; verso?: 'sx' | 'dx' | 'giu' }>(),
  { testa: '#6e56cf', verso: 'sx' },
)
</script>

<template>
  <svg
    v-if="verso === 'giu'"
    width="50"
    height="50"
    viewBox="0 0 52 52"
    class="spillo"
    aria-hidden="true"
  >
    <line x1="26" y1="46" x2="26" y2="14" stroke="#c7c4ba" stroke-width="2.6" stroke-linecap="round" />
    <circle cx="26" cy="12" r="7" :fill="testa" />
    <circle cx="23.5" cy="9.5" r="2.2" fill="rgba(255,255,255,.75)" />
  </svg>
  <svg
    v-else
    width="54"
    height="54"
    viewBox="0 0 52 52"
    class="spillo"
    :class="{ dx: verso === 'dx' }"
    aria-hidden="true"
  >
    <line x1="12" y1="42" x2="34" y2="18" stroke="#c7c4ba" stroke-width="2.6" stroke-linecap="round" />
    <circle cx="36" cy="16" r="7" :fill="testa" />
    <circle cx="33.5" cy="13.5" r="2.2" fill="rgba(255,255,255,.75)" />
  </svg>
</template>

<style scoped>
.spillo {
  filter: drop-shadow(0 4px 5px rgba(60, 50, 30, 0.35));
}
.spillo.dx {
  transform: scaleX(-1);
}
</style>
