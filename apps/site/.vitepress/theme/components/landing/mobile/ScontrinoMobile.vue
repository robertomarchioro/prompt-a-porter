<script setup lang="ts">
import Spillo from '../desktop/Spillo.vue'
</script>

<template>
  <div id="collezione-m" class="wrap">
    <Spillo class="spillo-sx" testa="#6e56cf" verso="sx" />
    <Spillo class="spillo-dx" testa="#d8a463" verso="dx" />
    <div class="scontrino">
      <div class="testata">
        <div class="titolo">Atelier · scontrino</div>
        <div class="numero">Collezione AW26 · Nº 3A-001</div>
      </div>
      <div class="voci">
        <div class="colonne"><span>Capo</span><span>Prezzo</span></div>
        <div class="voce">
          <span class="capo">
            <b>Ogni modifica, in vetrina</b>
            <span class="desc">save = versione · diff git · ripristino</span>
          </span>
          <span class="incl">incl.</span>
        </div>
        <div class="voce">
          <span class="capo">
            <b>Capi su misura</b>
            <span class="desc"><span class="pap-tok" v-pre>{{nome}}</span> ad-hoc · <span class="pap-tok" v-pre>{{globale autore}}</span></span>
          </span>
          <span class="incl">incl.</span>
        </div>
        <div class="voce">
          <span class="capo">
            <b>Cerca al buio dell'armadio</b>
            <span class="desc">significato, non parola · ONNX MiniLM</span>
          </span>
          <span class="incl">incl.</span>
        </div>
        <div class="voce">
          <span class="capo">
            <b>Prova prima di comprare</b>
            <span class="desc">varianti A/B · golden test · costo stimato</span>
          </span>
          <span class="incl">incl.</span>
        </div>
        <div class="voce">
          <span class="capo">
            <b>Un tono, dieci tagli</b>
            <span class="desc"><span class="pap-tok" v-pre>{{import "ruolo"}}</span> · sartoria modulare</span>
          </span>
          <span class="incl">incl.</span>
        </div>
        <div class="voce ultima">
          <span class="capo">
            <b>Il taglio da terminale</b>
            <span class="desc"><span class="viola">pap</span> render in pipe · sola lettura</span>
          </span>
          <span class="incl">incl.</span>
        </div>
      </div>
      <div class="servizi">
        <span class="slabel">Servizi dell'atelier · inclusi</span>
        <span class="chip">Ritocco ✦</span>
        <span class="chip">Diagnosi</span>
        <span class="chip">MCP</span>
      </div>
      <div class="totale-wrap">
        <div class="totale">
          <span class="tlabel">Totale</span>
          <span class="gratis">gratis</span>
        </div>
        <div class="tmeta">local-first · open source · AGPL 3.0</div>
      </div>
      <div class="dentellatura" aria-hidden="true"></div>
    </div>
  </div>
</template>

<style scoped>
.wrap {
  margin: 6px 16px 0;
  position: relative;
}
.spillo-sx {
  position: absolute;
  left: -10px;
  top: -14px;
  z-index: 3;
}
.spillo-dx {
  position: absolute;
  right: -10px;
  top: -14px;
  z-index: 3;
}
.scontrino {
  background: var(--pap-surface);
  border: 1px solid var(--pap-line);
  border-radius: 4px;
  box-shadow: 0 20px 44px -26px rgba(90, 88, 78, 0.5);
  position: relative;
}
.testata {
  text-align: center;
  padding: 20px 18px 12px;
  border-bottom: 1px dashed var(--pap-dash);
}
.testata .titolo {
  font-family: var(--pap-serif);
  font-weight: 400;
  font-size: 22px;
  letter-spacing: -0.01em;
}
.numero {
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--pap-faint);
  margin-top: 6px;
}
.voci {
  padding: 6px 18px;
}
.colonne {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--pap-faint);
  padding: 10px 0 8px;
  border-bottom: 1px solid var(--pap-ink);
}
.voce {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 0;
  border-bottom: 1px dashed var(--pap-line-soft2);
}
.voce.ultima {
  border-bottom: none;
}
.capo {
  flex: 1;
}
.capo b {
  font-weight: 600;
  font-size: 13.5px;
  color: var(--pap-ink);
}
.capo .desc {
  display: block;
  font-size: 11.5px;
  color: var(--pap-muted);
}
.capo .viola {
  color: var(--pap-viola);
}
.incl {
  font-family: var(--pap-mono);
  font-size: 11px;
  color: var(--pap-verde);
}
.servizi {
  margin: 0 18px;
  padding: 11px 0;
  border-top: 1px dashed var(--pap-dash);
  border-bottom: 1px dashed var(--pap-dash);
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.slabel {
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--pap-faint);
  width: 100%;
}
.chip {
  font-size: 11.5px;
  color: var(--pap-sub);
  background: var(--pap-page);
  border: 1px solid var(--pap-line);
  border-radius: 100px;
  padding: 3px 9px;
}
.totale-wrap {
  padding: 14px 18px;
}
.totale {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.tlabel {
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--pap-ink);
}
.gratis {
  font-family: var(--pap-serif);
  font-style: italic;
  font-size: 26px;
  color: var(--pap-viola);
}
.tmeta {
  font-size: 11px;
  color: var(--pap-faint);
  text-align: right;
  margin-top: 2px;
}
.dentellatura {
  height: 19px;
  background:
    radial-gradient(circle at 9px 19px, var(--pap-dash) 9px, transparent 9.5px),
    radial-gradient(circle at 9px 17px, var(--pap-page) 9px, transparent 9.5px);
  background-size: 18px 19px;
  background-repeat: repeat-x;
}
</style>
