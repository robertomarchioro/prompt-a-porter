<script setup lang="ts">
import { withBase } from 'vitepress'
import { GUIDA } from '../links'
import { useLinkDownload } from '../download'

const linkDownload = useLinkDownload()
</script>

<template>
  <div class="topbar">
    <div class="brand">
      <img class="mark" :src="withBase('/icons/icon-64.png')" alt="" width="30" height="30" />
      <span class="nome">Prompt a Porter</span>
    </div>
    <nav class="nav" aria-label="Navigazione principale">
      <a href="#collezione">La collezione</a>
      <a href="#servizi">I servizi</a>
      <a href="#come-funziona">Come funziona</a>
      <a :href="GUIDA">Guida</a>
      <a class="scarica" :href="linkDownload">Scarica</a>
    </nav>
  </div>
</template>

<style scoped>
.topbar {
  max-width: 1180px;
  margin: 0 auto;
  padding: 26px 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.brand {
  display: flex;
  align-items: center;
  gap: 12px;
}
/* Icona ufficiale { P } (docs/public/icons/), non il placeholder del
   prototipo — §2.3 istruzioni. */
.mark {
  width: 30px;
  height: 30px;
  display: block;
}
.nome {
  font-family: var(--pap-serif);
  font-size: 19px;
}
.nav {
  display: flex;
  align-items: center;
  gap: 30px;
}
.nav a {
  font-size: 13.5px;
  color: var(--pap-sub);
  padding: 5px 0;
}
.nav a:hover {
  color: var(--pap-viola);
}
.nav .scarica {
  color: var(--pap-ink);
  border: 1px solid var(--pap-line);
  background: var(--pap-surface);
  border-radius: 9px;
  padding: 9px 18px;
}
.nav .scarica:hover {
  color: var(--pap-viola);
  border-color: var(--pap-viola-chiaro);
}

@media (max-width: 1080px) {
  .topbar {
    padding: 22px 36px;
  }
  .nav {
    gap: 20px;
  }
}
</style>
