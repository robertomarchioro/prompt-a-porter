<script setup lang="ts">
import { useScorciatoie } from '../os'

const { compilaIncolla } = useScorciatoie()
</script>

<template>
  <div class="wrap">
    <div class="stage">
      <div class="documento">
        <div class="finestra">
          <span class="pallino" aria-hidden="true"></span>
          <span class="pallino" aria-hidden="true"></span>
          <span class="pallino" aria-hidden="true"></span>
          <span class="fmeta">documento</span>
        </div>
        <div class="riga r1" aria-hidden="true"></div>
        <div class="riga r2" aria-hidden="true"></div>
        <div class="riga r3" aria-hidden="true"></div>
      </div>
      <div class="palette">
        <div class="search">
          <svg viewBox="0 0 24 24" class="pap-mag lente" aria-hidden="true">
            <circle cx="11" cy="11" r="7" />
            <line x1="16.5" y1="16.5" x2="21" y2="21" />
          </svg>
          <span class="ph">Cerca prompt…</span>
          <span class="caret" aria-hidden="true"></span>
        </div>
        <div class="item attivo">
          <div class="corpo">
            <div class="titolo">Riassumi articolo in N punti</div>
            <div class="desc">Distilla in <em>N</em> punti + TL;DR</div>
          </div>
          <span class="badge">privato</span>
        </div>
        <div class="item">
          <div class="corpo">
            <div class="titolo">Email professionale parametrica</div>
            <div class="desc">Tono e contenuto variabili</div>
          </div>
          <span class="badge">privato</span>
        </div>
        <div class="hints">
          <span>↵ seleziona</span>
          <span class="viola">{{ compilaIncolla }} compila e incolla</span>
        </div>
      </div>
    </div>
    <div class="caption">Funzione in evidenza · La palette</div>
  </div>
</template>

<style scoped>
.stage {
  margin: 2px 20px 0;
  position: relative;
  border-radius: 22px;
  overflow: hidden;
  border: 1px solid var(--pap-line);
  box-shadow: 0 22px 50px -30px rgba(90, 88, 78, 0.5);
}
.documento {
  background: linear-gradient(160deg, #e7e5de, #d9d7ce);
  padding: 22px 18px 82px;
}
.finestra {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 14px;
}
.pallino {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #c9c6bb;
}
.fmeta {
  font-family: var(--pap-mono);
  font-size: 10px;
  color: #a9a69a;
  margin-left: 8px;
}
.riga {
  height: 8px;
  background: #d4d1c7;
  border-radius: 100px;
  margin-bottom: 9px;
}
.riga.r1 {
  width: 66%;
  background: #cbc8bd;
}
.riga.r2 {
  width: 84%;
}
.riga.r3 {
  width: 48%;
  margin-bottom: 0;
}
.palette {
  position: absolute;
  left: 16px;
  right: 16px;
  bottom: 14px;
  background: var(--pap-white);
  border: 1px solid var(--pap-line-soft);
  border-radius: 14px;
  box-shadow: 0 26px 46px -22px rgba(90, 88, 78, 0.55), 0 0 0 1px rgba(132, 112, 213, 0.12);
  overflow: hidden;
  text-align: left;
}
.search {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 13px 15px;
  border-bottom: 1px solid #f0eee8;
}
.lente {
  width: 15px;
  height: 15px;
  stroke: var(--pap-faint);
}
.ph {
  font-size: 13.5px;
  color: var(--pap-faint);
}
.caret {
  width: 2px;
  height: 14px;
  background: var(--pap-viola);
  margin-left: -4px;
  animation: pap-blink 1.1s steps(1) infinite;
}
.item {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 9px 15px;
}
.item.attivo {
  background: linear-gradient(90deg, rgba(132, 112, 213, 0.1), transparent);
  box-shadow: inset 2px 0 0 var(--pap-viola);
}
.corpo {
  flex: 1;
}
.titolo {
  font-size: 13.5px;
  font-weight: 500;
  color: var(--pap-ink);
}
.desc {
  font-size: 12px;
  color: var(--pap-muted);
  margin-top: 2px;
}
.desc em {
  font-style: normal;
  color: var(--pap-viola);
}
.badge {
  font-family: var(--pap-mono);
  font-size: 11px;
  color: var(--pap-ambra-testo);
  background: var(--pap-ambra-bg);
  border: 1px solid var(--pap-ambra-bordo);
  border-radius: 5px;
  padding: 3px 6px;
}
.hints {
  display: flex;
  gap: 12px;
  padding: 10px 15px;
  border-top: 1px solid #f0eee8;
  background: var(--pap-surface);
}
.hints span {
  font-size: 11px;
  color: var(--pap-faint);
}
.hints .viola {
  color: var(--pap-viola);
  margin-left: auto;
}
.caption {
  text-align: center;
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--pap-faint);
  padding: 16px 0 4px;
}
</style>
