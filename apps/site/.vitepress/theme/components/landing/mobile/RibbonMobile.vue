<script setup lang="ts">
import { RELEASES } from '../links'
</script>

<template>
  <div class="ribbon">
    <div class="righe" aria-hidden="true"></div>
    <div class="alone" aria-hidden="true"></div>
    <div class="rin">
      <div class="testa">
        <span class="flag">✦ Nuova stagione</span>
        <span class="aw">AW 2026 · v1.0</span>
      </div>
      <div class="titolo">Debutta <em>Arioso Atelier</em></div>
      <div class="riga">
        <span class="msg">
          La prima stagione esce dalla beta <span class="dot">·</span>
        </span>
        <a class="cta" :href="RELEASES">Scopri il debutto →</a>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ribbon {
  margin: 12px 14px 0;
  border-radius: 16px;
  position: relative;
  overflow: hidden;
  background: linear-gradient(135deg, var(--pap-viola2), var(--pap-viola) 52%, var(--pap-viola-deep));
  box-shadow: 0 18px 38px -18px rgba(110, 86, 207, 0.65);
}
.righe {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(115deg, rgba(255, 255, 255, 0.09) 0 2px, transparent 2px 22px);
  pointer-events: none;
}
.alone {
  position: absolute;
  right: -26px;
  top: -26px;
  width: 120px;
  height: 120px;
  background: radial-gradient(circle, rgba(232, 199, 154, 0.45), transparent 66%);
  pointer-events: none;
}
.rin {
  position: relative;
  padding: 14px 16px 15px;
}
.testa {
  display: flex;
  align-items: center;
  gap: 9px;
}
.flag {
  font-family: var(--pap-mono);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  font-weight: 700;
  color: #3a2a12;
  background: var(--pap-ambra-flag);
  border-radius: 100px;
  padding: 4px 10px;
}
.aw {
  font-family: var(--pap-mono);
  font-size: 10px;
  letter-spacing: 0.1em;
  color: #e4defa;
  margin-left: auto;
}
.titolo {
  font-family: var(--pap-serif);
  font-weight: 300;
  font-size: 26px;
  line-height: 1.02;
  color: #fff;
  margin: 11px 0 0;
}
.titolo em {
  font-style: italic;
  color: var(--pap-crema);
}
.riga {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}
.msg {
  font-size: 11px;
  color: #e4defa;
  flex: 1;
  line-height: 1.35;
}
.dot {
  color: var(--pap-viola-chiaro);
}
.codice {
  font-family: var(--pap-mono);
  font-size: 10px;
  color: var(--pap-crema);
}
.cta {
  font-size: 11px;
  font-weight: 600;
  color: #fff;
  background: rgba(255, 255, 255, 0.16);
  border: 1px solid rgba(255, 255, 255, 0.28);
  border-radius: 100px;
  padding: 6px 12px;
  white-space: nowrap;
}
.cta:hover {
  background: rgba(255, 255, 255, 0.26);
}
</style>
