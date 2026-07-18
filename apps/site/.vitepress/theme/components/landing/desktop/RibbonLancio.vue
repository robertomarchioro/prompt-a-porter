<script setup lang="ts">
import { RELEASES } from '../links'
</script>

<template>
  <div class="ribbon">
    <div class="righe" aria-hidden="true"></div>
    <div class="rin">
      <span class="flag">✦ Nuova stagione</span>
      <span class="msg">
        Debutta <em>Arioso Atelier</em> · la prima stagione esce dalla beta
        <span class="dot">·</span> Autunno-Inverno 2026 · v1.0 <span class="dot">·</span> codice
        <span class="codice">Cloud Dancer 11-4201</span>
      </span>
      <a class="cta" :href="RELEASES">Scopri il debutto →</a>
    </div>
  </div>
</template>

<style scoped>
.ribbon {
  position: relative;
  overflow: hidden;
  background: linear-gradient(100deg, var(--pap-viola2), var(--pap-viola) 50%, var(--pap-viola-deep));
  color: #fff;
}
.righe {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(115deg, rgba(255, 255, 255, 0.08) 0 2px, transparent 2px 24px);
  pointer-events: none;
}
.rin {
  position: relative;
  max-width: 1180px;
  margin: 0 auto;
  padding: 11px 48px;
  display: flex;
  align-items: center;
  gap: 18px;
}
.flag {
  font-family: var(--pap-mono);
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  font-weight: 700;
  color: #3a2a12;
  background: var(--pap-ambra-flag);
  border-radius: 100px;
  padding: 4px 11px;
  flex: none;
}
.msg {
  font-size: 13px;
  color: #e4defa;
  flex: 1;
}
.msg em {
  font-family: var(--pap-serif);
  font-style: italic;
  color: var(--pap-crema);
}
.dot {
  color: var(--pap-viola-chiaro);
}
.codice {
  font-family: var(--pap-mono);
  font-size: 11px;
  color: var(--pap-crema);
}
.cta {
  font-size: 12.5px;
  font-weight: 600;
  color: #fff;
  white-space: nowrap;
  flex: none;
}
.cta:hover {
  color: var(--pap-crema);
}

@media (max-width: 1080px) {
  .rin {
    padding: 11px 36px;
  }
}
</style>
