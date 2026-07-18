<template>
  <div class="banda">
    <div class="alone" aria-hidden="true"></div>
    <div class="grid">
      <div>
        <div class="kicker">— manifesto —</div>
        <div class="slogan">
          Se ci puoi scrivere, ci puoi <em>incollare il tuo prompt.</em>
        </div>
        <div class="tecnica">
          Si abbina con qualsiasi campo di testo, in ogni editor, applicazione o sito web.
        </div>
      </div>
      <div class="arte">
        <div class="alone2" aria-hidden="true"></div>
        <div class="mini-palette">
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
              <div class="titolo">Brainstorm con criteri</div>
              <div class="desc">Idee valutate su criteri scelti da te</div>
            </div>
            <span class="badge">privato</span>
          </div>
          <div class="item">
            <div class="corpo">
              <div class="titolo">Traduzione con glossario</div>
              <div class="desc">Rispetta i termini che scegli</div>
            </div>
            <span class="badge">privato</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.banda {
  margin: 34px 0 0;
  padding: 70px 48px;
  background: #edebf6;
  border-top: 1px solid var(--pap-line);
  border-bottom: 1px solid var(--pap-line);
  position: relative;
  overflow: hidden;
}
.alone {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse 60% 120% at 50% 50%, rgba(132, 112, 213, 0.13), transparent 72%);
  pointer-events: none;
}
.grid {
  position: relative;
  max-width: 1080px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: 1.02fr 0.98fr;
  gap: 56px;
  align-items: center;
}
.kicker {
  font-family: var(--pap-mono);
  font-size: 12px;
  letter-spacing: 0.3em;
  text-transform: uppercase;
  color: var(--pap-viola);
  margin-bottom: 18px;
}
.slogan {
  font-family: var(--pap-serif);
  font-weight: 300;
  font-size: 56px;
  line-height: 1.03;
  letter-spacing: -0.025em;
  color: var(--pap-ink);
}
.slogan em {
  font-style: italic;
  color: var(--pap-viola);
}
.tecnica {
  font-family: var(--pap-mono);
  font-size: 15px;
  line-height: 1.6;
  letter-spacing: 0.03em;
  color: var(--pap-muted);
  margin-top: 26px;
  padding-top: 22px;
  border-top: 1px solid #dddace;
  max-width: 440px;
}
.arte {
  display: flex;
  justify-content: center;
  position: relative;
}
.alone2 {
  position: absolute;
  width: 80%;
  height: 80%;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  background: radial-gradient(ellipse 60% 60% at 50% 50%, rgba(132, 112, 213, 0.28), transparent 70%);
  filter: blur(14px);
  pointer-events: none;
}
.mini-palette {
  position: relative;
  width: 420px;
  transform: rotate(-2deg);
  background: var(--pap-white);
  border: 1px solid var(--pap-line-soft);
  border-radius: 16px;
  box-shadow: 0 40px 80px -34px rgba(90, 88, 78, 0.5);
  overflow: hidden;
  text-align: left;
}
.search {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 14px 17px;
  border-bottom: 1px solid #f0eee8;
}
.lente {
  width: 15px;
  height: 15px;
  stroke: var(--pap-faint);
}
.ph {
  font-size: 13px;
  color: var(--pap-faint);
}
.caret {
  width: 2px;
  height: 15px;
  background: var(--pap-viola);
  margin-left: -4px;
  animation: pap-blink 1.1s steps(1) infinite;
}
.item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 17px;
}
.item.attivo {
  background: #f2ecde;
  box-shadow: inset 2px 0 0 var(--pap-viola);
}
.corpo {
  flex: 1;
}
.titolo {
  font-size: 13px;
  font-weight: 500;
  color: var(--pap-ink);
}
.desc {
  font-size: 11px;
  color: var(--pap-muted);
  margin-top: 2px;
}
.badge {
  font-family: var(--pap-mono);
  font-size: 10px;
  color: var(--pap-ambra-testo);
  background: #efe3ce;
  border: 1px solid #e2d2b4;
  border-radius: 5px;
  padding: 3px 7px;
}

@media (max-width: 1080px) {
  .banda {
    padding: 56px 36px;
  }
  .slogan {
    font-size: clamp(40px, 5.5vw, 56px);
  }
}

@media (max-width: 900px) {
  .grid {
    grid-template-columns: 1fr;
    gap: 44px;
  }
}
</style>
