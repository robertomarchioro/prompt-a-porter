<script setup lang="ts">
import { GITHUB } from '../links'
import { useLinkDownload } from '../download'
import { useNomeOsVisitatore, useScorciatoie } from '../os'
import { traccia } from '../analytics'
import IconOs from './IconOs.vue'

const linkDownload = useLinkDownload()
const nomeOs = useNomeOsVisitatore()
const { paletta } = useScorciatoie()

function clickDownload() {
  traccia('download', 'hero', nomeOs.value ?? 'sconosciuto')
}
</script>

<template>
  <div class="hero">
    <div class="glow" aria-hidden="true"></div>
    <div class="in">
      <span class="eyebrow">
        <span class="kbd">{{ paletta }}</span>
        <span class="etesto">richiama la tua collezione, in qualsiasi app</span>
      </span>
      <h1>Prompt-<em>à</em>-porter</h1>
      <p class="sub">
        La tua collezione di prompt AI. Versionati, su misura,
        <em>a un tasto di distanza</em> — ovunque tu stia scrivendo.
      </p>
      <div class="cta-row">
        <a class="pap-btn-primary" :href="linkDownload" @click="clickDownload">↓ Scarica l'app</a>
        <a class="pap-btn-ghost" :href="GITHUB">Vedi su GitHub</a>
      </div>
      <div class="piattaforme">
        <span class="os"><IconOs os="windows" />Windows</span>
        <span class="os"><IconOs os="macos" />macOS</span>
        <span class="os"><IconOs os="linux" />Linux</span>
        <span class="meta">gratis · local-first · v1.0</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hero {
  max-width: 1180px;
  margin: 0 auto;
  padding: 42px 48px 24px;
  text-align: center;
  position: relative;
}
.glow {
  position: absolute;
  left: 50%;
  top: 0;
  transform: translateX(-50%);
  width: 900px;
  height: 520px;
  background: radial-gradient(ellipse 50% 46% at 50% 28%, rgba(132, 112, 213, 0.16), transparent 70%);
  pointer-events: none;
}
.in {
  position: relative;
}
.eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 9px;
  background: var(--pap-surface);
  border: 1px solid var(--pap-line);
  border-radius: 100px;
  padding: 7px 16px 7px 9px;
  margin-bottom: 24px;
}
.kbd {
  font-family: var(--pap-mono);
  font-size: 11px;
  background: var(--pap-white);
  border: 1px solid var(--pap-line);
  border-bottom-width: 2px;
  border-radius: 5px;
  padding: 3px 7px;
  color: var(--pap-viola);
}
.etesto {
  font-size: 13px;
  color: var(--pap-muted);
}
h1 {
  font-family: var(--pap-serif);
  font-weight: 300;
  font-size: 98px;
  line-height: 0.92;
  letter-spacing: -0.04em;
  margin: 0;
}
h1 em {
  font-style: italic;
  color: var(--pap-viola);
}
.sub {
  font-family: var(--pap-serif);
  /* 400 sotto i 30px: il 300 su fondo chiaro degrada la leggibilità
     (decisione 2026-07-18, vale per tutti i serif medi). */
  font-weight: 400;
  font-size: 25px;
  line-height: 1.4;
  color: var(--pap-sub);
  margin: 24px auto 0;
  max-width: 600px;
}
.sub em {
  font-style: italic;
  color: var(--pap-ink);
}
.cta-row {
  display: flex;
  gap: 14px;
  justify-content: center;
  align-items: center;
  margin-top: 32px;
}
.piattaforme {
  display: flex;
  gap: 22px;
  justify-content: center;
  align-items: center;
  margin-top: 20px;
}
.os {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 13px;
  color: var(--pap-sub);
}
.meta {
  font-family: var(--pap-mono);
  font-size: 11.5px;
  color: var(--pap-faint);
}

@media (max-width: 1080px) {
  .hero {
    padding: 42px 36px 24px;
  }
  h1 {
    font-size: clamp(56px, 8.5vw, 98px);
  }
}
</style>
