<script setup lang="ts">
// Icone piattaforma monocrome (handoff): Windows 4 quadrati, mela macOS,
// Tux con becco/zampe ambra. Decorative: il nome OS è testo adiacente.
defineProps<{ os: 'windows' | 'macos' | 'linux' }>()
</script>

<template>
  <svg v-if="os === 'windows'" viewBox="0 0 24 24" class="ios ios-win" aria-hidden="true">
    <g fill="currentColor">
      <rect x="2" y="2" width="8.6" height="8.6" />
      <rect x="13.4" y="2" width="8.6" height="8.6" />
      <rect x="2" y="13.4" width="8.6" height="8.6" />
      <rect x="13.4" y="13.4" width="8.6" height="8.6" />
    </g>
  </svg>
  <svg v-else-if="os === 'macos'" viewBox="0 0 24 24" class="ios ios-mac" aria-hidden="true">
    <path
      fill="currentColor"
      d="M17.6 12.7c0-2.3 1.9-3.4 2-3.5-1.1-1.6-2.8-1.8-3.4-1.8-1.4-.15-2.8.85-3.5.85-.7 0-1.85-.83-3.05-.8-1.55.02-3 .9-3.8 2.3-1.65 2.85-.42 7.05 1.17 9.36.78 1.13 1.7 2.4 2.9 2.35 1.17-.05 1.6-.75 3.02-.75 1.4 0 1.8.75 3.05.72 1.26-.02 2.06-1.15 2.83-2.28.9-1.3 1.27-2.57 1.29-2.63-.03-.01-2.47-.95-2.5-3.75zM15.3 6.1c.64-.78 1.07-1.86.95-2.94-.92.04-2.04.61-2.7 1.39-.59.69-1.11 1.8-.97 2.85 1.03.08 2.08-.52 2.72-1.3z"
    />
  </svg>
  <svg v-else viewBox="0 0 24 24" class="ios ios-tux" aria-hidden="true">
    <ellipse cx="12" cy="13" rx="7" ry="8.6" fill="currentColor" />
    <ellipse cx="12" cy="14.2" rx="4.2" ry="6.2" fill="#f5f4f0" />
    <circle cx="9.5" cy="7.2" r="1.1" fill="#f5f4f0" />
    <circle cx="14.5" cy="7.2" r="1.1" fill="#f5f4f0" />
    <path d="M10.6 8.7 L13.4 8.7 L12 10.5 Z" fill="#d8a463" />
    <path d="M9 20.4 C7.6 21.6 6.2 21.4 6 20.4 6.4 19.2 7.8 19 9 19.2 Z" fill="#d8a463" />
    <path d="M15 20.4 C16.4 21.6 17.8 21.4 18 20.4 17.6 19.2 16.2 19 15 19.2 Z" fill="#d8a463" />
  </svg>
</template>

<style scoped>
.ios {
  color: var(--pap-sub);
}
.ios-win {
  width: 13px;
  height: 13px;
}
.ios-mac {
  width: 12px;
  height: 13px;
}
.ios-tux {
  width: 11px;
  height: 14px;
}
</style>
