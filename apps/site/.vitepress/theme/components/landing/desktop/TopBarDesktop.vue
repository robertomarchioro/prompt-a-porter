<script setup lang="ts">
import { GUIDA } from '../links'
import { useLinkDownload } from '../download'

const linkDownload = useLinkDownload()
</script>

<template>
  <div class="topbar">
    <div class="brand">
      <span class="mark" aria-hidden="true">P</span>
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
.mark {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: linear-gradient(150deg, var(--pap-viola2), var(--pap-viola));
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--pap-serif);
  font-size: 18px;
  color: #fff;
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
