<script setup lang="ts">
import { RELEASES } from '../links'
import { traccia } from '../analytics'
import IconOs from '../desktop/IconOs.vue'

// Fase A (§5.3 istruzioni): il form «Mandami il link» arriva con F4
// (endpoint su Giganto + listmonk). Fino ad allora la CTA degrada a un
// link diretto alla pagina di download — la sezione non resta muta.
</script>

<template>
  <div id="come-funziona-m" class="card">
    <div class="alone" aria-hidden="true"></div>
    <div class="in">
      <div class="kicker">L'atelier è sul desktop</div>
      <div class="titolo">
        Questo capo si prova<br />al computer. <em>Te lo mandiamo.</em>
      </div>
      <p class="testo">
        Prompt a Porter vive su Windows, macOS e Linux. Il link ti aspetta quando ti siedi al
        computer. Niente da inquadrare, niente app da cercare.
      </p>
      <div class="cta-row">
        <a class="cta" :href="RELEASES" @click="traccia('download', 'mobile-fallback')">Apri la pagina di download →</a>
      </div>
      <p class="nota">
        <span class="pallino" aria-hidden="true"></span>
        un solo link. Niente newsletter, niente account, niente scuse.
      </p>
      <div class="piattaforme">
        <a :href="RELEASES" @click="traccia('download', 'mobile-piattaforme', 'windows')"><IconOs os="windows" />Windows</a>
        <a :href="RELEASES" @click="traccia('download', 'mobile-piattaforme', 'macos')"><IconOs os="macos" />macOS</a>
        <a :href="RELEASES" @click="traccia('download', 'mobile-piattaforme', 'linux')"><IconOs os="linux" />Linux</a>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  margin: 22px 16px 0;
  background: linear-gradient(165deg, #f4f1fc, #ede9f7 62%, #f3eee6);
  color: var(--pap-ink);
  border-radius: 20px;
  padding: 22px 20px;
  position: relative;
  overflow: hidden;
  border: 1px solid #e1dbf2;
}
.alone {
  position: absolute;
  right: -34px;
  top: -34px;
  width: 160px;
  height: 160px;
  background: radial-gradient(circle, rgba(132, 112, 213, 0.22), transparent 66%);
  pointer-events: none;
}
.in {
  position: relative;
}
.kicker {
  font-family: var(--pap-mono);
  font-size: 12.5px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--pap-viola);
}
.titolo {
  font-family: var(--pap-serif);
  font-weight: 400;
  font-size: 27px;
  line-height: 1.08;
  margin-top: 10px;
}
.titolo em {
  font-style: italic;
  color: var(--pap-viola);
}
.testo {
  font-size: 15px;
  line-height: 1.55;
  color: var(--pap-muted);
  margin: 12px 0 0;
}
.cta-row {
  display: flex;
  margin-top: 16px;
}
.cta {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
  font-weight: 600;
  padding: 13px 18px;
  border-radius: 12px;
  background: linear-gradient(160deg, var(--pap-viola2), var(--pap-viola));
  color: #fff;
  box-shadow: 0 12px 26px -12px rgba(132, 112, 213, 0.7);
  transition: filter 0.18s ease;
  min-height: 44px;
}
.cta:hover {
  filter: brightness(1.08);
}
.nota {
  font-size: 13px;
  color: var(--pap-muted);
  margin: 12px 0 0;
  display: flex;
  align-items: center;
  gap: 7px;
}
.pallino {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--pap-verde);
  flex: none;
}
.piattaforme {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid #e1dbf2;
}
.piattaforme a {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13.5px;
  color: var(--pap-sub);
  padding: 8px 0;
}
.piattaforme a:hover {
  color: var(--pap-viola);
}
</style>
