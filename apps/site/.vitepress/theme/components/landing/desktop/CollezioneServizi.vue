<script setup lang="ts">
import Spillo from './Spillo.vue'
</script>

<template>
  <div id="collezione" class="banda">
    <div class="grid">
      <!-- Scontrino come colonna laterale, spillato -->
      <div class="scontrino-wrap">
        <Spillo class="spillo-sx" testa="#6e56cf" verso="sx" />
        <Spillo class="spillo-dx" testa="#d8a463" verso="dx" />
        <div class="scontrino">
          <div class="testata">
            <div class="titolo">La collezione · scontrino</div>
            <div class="numero">Sei capi · AW26 · Nº 4A-001</div>
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

      <!-- I servizi dell'atelier -->
      <div id="servizi" class="servizi">
        <div class="kicker">I servizi dell'atelier</div>
        <h2>La collezione è tua.<br />L'atelier <em>la tiene in forma.</em></h2>
        <p class="raccordo">
          La controlla, la ritocca, te la porge ovunque tu stia lavorando — in un'altra app, nel
          terminale, dentro il tuo assistente.
        </p>
        <div class="cards">
          <div class="card">
            <div class="chead"><span class="cicona">✦</span><span class="cnome">Ritocco</span></div>
            <p>Un provider AI a tua scelta propone migliorie a diff, tarate sul modello. Accetti → nuova versione, storico intatto.</p>
          </div>
          <div class="card">
            <div class="chead"><span class="cicona">◉</span><span class="cnome">Diagnosi</span></div>
            <p>Un linter dei prompt segnala segnaposti malformati e fragilità. Ogni regola si attiva e si tara da te.</p>
          </div>
          <div class="card">
            <div class="chead"><span class="cicona cli">›_</span><span class="cnome">CLI <span class="cmono">pap</span></span></div>
            <p>Stesso vault, in sola lettura: <span class="cmd">pap render --var</span> e via in pipe. Per chi vive nella shell.</p>
          </div>
          <div class="card">
            <div class="chead"><span class="cicona">⊕</span><span class="cnome">MCP</span></div>
            <p>Espone il vault a Claude Desktop, Cursor e ogni client MCP. Solo via stdio: nessuna porta aperta.</p>
          </div>
        </div>
        <svg viewBox="0 0 560 30" class="filo" aria-hidden="true">
          <path
            d="M4 20 C 70 6, 140 6, 206 20 S 340 34, 406 20 S 520 8, 556 18"
            fill="none"
            stroke="#c89a55"
            stroke-width="1.6"
            stroke-dasharray="7 6"
            stroke-linecap="round"
          />
          <g transform="translate(556 18) rotate(20)">
            <path d="M0 0 L 20 -2 L 20 2 Z" fill="#b9b6ac" />
            <line x1="20" y1="0" x2="30" y2="0" stroke="#c7c4ba" stroke-width="2.4" stroke-linecap="round" />
          </g>
        </svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.banda {
  max-width: 1080px;
  margin: 0 auto;
  padding: 80px 48px 40px;
}
.grid {
  display: grid;
  grid-template-columns: 400px 1fr;
  gap: 56px;
  align-items: start;
}
.scontrino-wrap {
  position: relative;
}
.spillo-sx {
  position: absolute;
  left: -12px;
  top: -16px;
  z-index: 3;
}
.spillo-dx {
  position: absolute;
  right: -12px;
  top: -16px;
  z-index: 3;
}
.scontrino {
  background: var(--pap-surface);
  border: 1px solid var(--pap-line);
  border-radius: 4px;
  box-shadow: 0 30px 60px -34px rgba(90, 88, 78, 0.5);
}
.testata {
  text-align: center;
  padding: 22px 20px 13px;
  border-bottom: 1px dashed var(--pap-dash);
}
.testata .titolo {
  font-family: var(--pap-serif);
  font-weight: 400;
  font-size: 24px;
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
  padding: 6px 20px;
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
  padding: 11px 0 9px;
  border-bottom: 1px solid var(--pap-ink);
}
.voce {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  padding: 11px 0;
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
  font-size: 14px;
  color: var(--pap-ink);
}
.capo .desc {
  display: block;
  font-size: 12px;
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
.totale-wrap {
  padding: 15px 20px;
}
.totale {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 2px solid var(--pap-ink);
  padding-top: 12px;
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
  font-size: 28px;
  color: var(--pap-viola);
}
.tmeta {
  font-size: 11px;
  color: var(--pap-faint);
  text-align: right;
  margin-top: 2px;
}
.dentellatura {
  height: 20px;
  background:
    radial-gradient(circle at 10px 20px, var(--pap-dash) 10px, transparent 10.5px),
    radial-gradient(circle at 10px 18px, var(--pap-page) 10px, transparent 10.5px);
  background-size: 20px 20px;
  background-repeat: repeat-x;
}
.servizi .kicker {
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--pap-viola);
}
.servizi h2 {
  font-family: var(--pap-serif);
  font-weight: 300;
  font-size: 44px;
  line-height: 1.04;
  letter-spacing: -0.02em;
  margin: 14px 0 12px;
  border: none;
  padding: 0;
}
.servizi h2 em {
  font-style: italic;
  color: var(--pap-viola);
}
.raccordo {
  font-size: 16px;
  line-height: 1.65;
  color: var(--pap-sub);
  max-width: 460px;
  margin: 0 0 26px;
}
.cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.card {
  background: var(--pap-surface);
  border: 1px solid var(--pap-line);
  border-radius: 13px;
  padding: 18px 18px 20px;
}
.chead {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-bottom: 9px;
}
.cicona {
  width: 26px;
  height: 26px;
  border-radius: 7px;
  background: var(--pap-tint);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--pap-viola);
  font-size: 13px;
}
.cicona.cli {
  font-size: 12px;
}
.cnome {
  font-family: var(--pap-serif);
  font-size: 21px;
}
.cmono {
  font-family: var(--pap-mono);
  font-size: 14px;
  color: var(--pap-viola);
}
.card p {
  font-size: 14px;
  line-height: 1.5;
  color: var(--pap-muted);
  margin: 0;
}
.cmd {
  font-family: var(--pap-mono);
  font-size: 12px;
  color: var(--pap-sub);
}
.filo {
  width: 100%;
  height: 26px;
  margin-top: 20px;
  overflow: visible;
}

@media (max-width: 1080px) {
  .banda {
    padding: 64px 36px 32px;
  }
}

/* ≤900: scontrino a larghezza piena sopra i servizi (handoff §Responsive) */
@media (max-width: 900px) {
  .grid {
    grid-template-columns: 1fr;
    gap: 48px;
  }
  .scontrino-wrap {
    max-width: 520px;
    margin: 0 auto;
    width: 100%;
  }
}
</style>
