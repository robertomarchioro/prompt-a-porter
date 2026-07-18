<script setup lang="ts">
import { useLinkDownload } from '../download'
import IconOs from './IconOs.vue'
import Spillo from './Spillo.vue'

const linkDownload = useLinkDownload()
</script>

<template>
  <div id="come-funziona" class="banda">
    <div class="alone" aria-hidden="true"></div>
    <div class="grid">
      <div>
        <div class="kicker">In due minuti</div>
        <h2>Come provare <em>un abito.</em></h2>
        <p class="testo">
          Installer firmato per Windows, <code>.dmg</code> universale per macOS (Apple Silicon +
          Intel), <code>.deb</code>/AppImage per Linux. Aggiornamenti firmati, automatici e
          silenziosi.
        </p>
        <div class="cta-row">
          <a class="pap-btn-primary" :href="linkDownload">↓ Scarica la 1.0</a>
          <span class="meta">gratis · local-first · AGPL 3.0</span>
        </div>
        <div class="piattaforme">
          <span class="os"><IconOs os="windows" />Windows</span>
          <span class="os"><IconOs os="macos" />macOS <span class="badge">universale</span></span>
          <span class="os"><IconOs os="linux" />Linux</span>
        </div>
      </div>
      <div class="ticket-wrap">
        <Spillo class="spillo" testa="#6e56cf" verso="giu" />
        <div class="ticket">
          <div class="tlabel">Etichetta di manutenzione</div>
          <div class="righe">
            <span><span class="pallino" aria-hidden="true"></span>100% locale</span>
            <span><span class="pallino" aria-hidden="true"></span>no cloud</span>
            <span><span class="pallino" aria-hidden="true"></span>nessun account</span>
            <span><span class="pallino" aria-hidden="true"></span>lavabile in git</span>
            <span><span class="pallino" aria-hidden="true"></span>AGPL 3.0</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.banda {
  background: linear-gradient(165deg, #f4f1fc, #ede9f7 60%, #f3eee6);
  border-top: 1px solid #e1dbf2;
  position: relative;
  overflow: hidden;
}
.alone {
  position: absolute;
  right: -40px;
  top: -40px;
  width: 220px;
  height: 220px;
  background: radial-gradient(circle, rgba(132, 112, 213, 0.2), transparent 66%);
  pointer-events: none;
}
.grid {
  position: relative;
  max-width: 1080px;
  margin: 0 auto;
  padding: 64px 48px;
  display: grid;
  grid-template-columns: 1.1fr 0.9fr;
  gap: 56px;
  align-items: center;
}
.kicker {
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--pap-viola);
}
h2 {
  font-family: var(--pap-serif);
  font-weight: 300;
  font-size: 50px;
  line-height: 1.02;
  letter-spacing: -0.03em;
  margin: 14px 0 0;
  border: none;
  padding: 0;
}
h2 em {
  font-style: italic;
  color: var(--pap-viola);
}
.testo {
  font-size: 14.5px;
  line-height: 1.65;
  color: var(--pap-sub);
  max-width: 420px;
  margin: 16px 0 0;
}
.testo code {
  font-family: var(--pap-mono);
  font-size: 12.5px;
  background: none;
  padding: 0;
  color: inherit;
}
.cta-row {
  display: flex;
  gap: 14px;
  align-items: center;
  margin-top: 28px;
}
.meta {
  font-family: var(--pap-mono);
  font-size: 11px;
  color: var(--pap-muted);
}
.piattaforme {
  display: flex;
  gap: 22px;
  margin-top: 22px;
}
.os {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12.5px;
  color: var(--pap-sub);
}
.badge {
  font-family: var(--pap-mono);
  font-size: 10px;
  color: var(--pap-ambra-testo);
  background: var(--pap-ambra-bg);
  border: 1px solid var(--pap-ambra-bordo);
  border-radius: 100px;
  padding: 2px 7px;
  margin-left: 3px;
}
.ticket-wrap {
  position: relative;
  justify-self: center;
}
.spillo {
  position: absolute;
  left: 50%;
  top: -18px;
  transform: translateX(-50%);
  z-index: 3;
}
.ticket {
  width: 280px;
  background: var(--pap-surface);
  border: 1px solid var(--pap-line);
  border-radius: 9px;
  padding: 22px 22px 20px;
  box-shadow: 0 26px 54px -30px rgba(90, 88, 78, 0.5);
  transform: rotate(1.5deg);
}
.tlabel {
  font-family: var(--pap-mono);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--pap-faint);
  border-bottom: 1px dashed var(--pap-dash);
  padding-bottom: 10px;
}
.righe {
  display: flex;
  flex-direction: column;
  gap: 9px;
  margin-top: 14px;
}
.righe span {
  font-family: var(--pap-mono);
  font-size: 11px;
  color: var(--pap-sub);
  display: flex;
  align-items: center;
  gap: 8px;
}
.pallino {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--pap-ambra2);
}

@media (max-width: 1080px) {
  .grid {
    padding: 56px 36px;
  }
  h2 {
    font-size: clamp(38px, 5vw, 50px);
  }
}

@media (max-width: 900px) {
  .grid {
    grid-template-columns: 1fr;
    gap: 44px;
  }
  .ticket-wrap {
    justify-self: start;
    margin-top: 10px;
  }
}
</style>
