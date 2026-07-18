<template>
  <div class="wrap">
    <div class="stage">
      <div class="documento">
        <div class="finestra">
          <span class="pallino" aria-hidden="true"></span>
          <span class="pallino" aria-hidden="true"></span>
          <span class="pallino" aria-hidden="true"></span>
          <span class="fmeta">documento · qualsiasi editor</span>
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
          <span class="ph">Cerca prompt, tag o azione…</span>
          <span class="caret" aria-hidden="true"></span>
          <span class="esc">esc</span>
        </div>
        <div class="label">Recenti</div>
        <div class="item attivo">
          <div class="corpo">
            <div class="titolo">Riassumi articolo in N punti</div>
            <div class="desc">Distilla un testo lungo in <em>N</em> punti chiave + TL;DR.</div>
          </div>
          <span class="badge">privato</span>
        </div>
        <div class="item">
          <div class="corpo">
            <div class="titolo">Code review strutturata</div>
            <div class="desc">Review per severità. Importa il ruolo <em>engineer</em>.</div>
          </div>
          <span class="badge">privato</span>
        </div>
        <div class="item">
          <div class="corpo">
            <div class="titolo">Email professionale parametrica</div>
            <div class="desc">Email con tono e contenuto variabili.</div>
          </div>
          <span class="badge">privato</span>
        </div>
        <div class="hints">
          <span>esc chiudi</span>
          <span>↑↓ naviga</span>
          <span>↵ seleziona</span>
          <span class="viola">⌃↵ compila e incolla</span>
        </div>
      </div>
    </div>
    <div class="caption">Funzione in evidenza · La palette — premi <span class="viola">⌃⇧P</span> ovunque</div>
  </div>
</template>

<style scoped>
.stage {
  max-width: 1080px;
  margin: 26px auto 0;
  position: relative;
  border-radius: 20px;
  overflow: hidden;
  border: 1px solid var(--pap-line);
  box-shadow: 0 40px 90px -46px rgba(90, 88, 78, 0.6);
}
.documento {
  background: linear-gradient(160deg, #e7e5de, #d9d7ce);
  padding: 30px 34px 130px;
}
.finestra {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 20px;
}
.pallino {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #c9c6bb;
}
.fmeta {
  font-family: var(--pap-mono);
  font-size: 11px;
  color: #a9a69a;
  margin-left: 10px;
}
.riga {
  height: 10px;
  background: #d4d1c7;
  border-radius: 100px;
  margin-bottom: 12px;
}
.riga.r1 {
  width: 52%;
  background: #cbc8bd;
}
.riga.r2 {
  width: 74%;
}
.riga.r3 {
  width: 40%;
  margin-bottom: 0;
}
.palette {
  position: absolute;
  left: 50%;
  bottom: 26px;
  transform: translateX(-50%);
  width: 640px;
  max-width: 88%;
  background: var(--pap-white);
  border: 1px solid var(--pap-line-soft);
  border-radius: 16px;
  box-shadow: 0 40px 70px -26px rgba(90, 88, 78, 0.6), 0 0 0 1px rgba(132, 112, 213, 0.12);
  overflow: hidden;
  text-align: left;
}
.search {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid #f0eee8;
}
.lente {
  width: 17px;
  height: 17px;
  stroke: var(--pap-faint);
}
.ph {
  font-size: 15px;
  color: var(--pap-faint);
}
.caret {
  width: 2px;
  height: 17px;
  background: var(--pap-viola);
  margin-left: -4px;
  animation: pap-blink 1.1s steps(1) infinite;
}
.esc {
  margin-left: auto;
  font-family: var(--pap-mono);
  font-size: 10px;
  color: var(--pap-faint);
  border: 1px solid var(--pap-line);
  border-radius: 5px;
  padding: 3px 7px;
}
.label {
  font-family: var(--pap-mono);
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--pap-faint);
  padding: 12px 20px 6px;
}
.item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 11px 20px;
}
.item.attivo {
  background: linear-gradient(90deg, rgba(132, 112, 213, 0.1), transparent);
  box-shadow: inset 2px 0 0 var(--pap-viola);
}
.corpo {
  flex: 1;
}
.titolo {
  font-size: 14px;
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
  font-size: 10px;
  color: var(--pap-ambra-testo);
  background: var(--pap-ambra-bg);
  border: 1px solid var(--pap-ambra-bordo);
  border-radius: 5px;
  padding: 3px 8px;
}
.hints {
  display: flex;
  gap: 20px;
  padding: 13px 20px;
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
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--pap-faint);
  padding: 18px 0 6px;
}
.caption .viola {
  color: var(--pap-viola);
}
</style>
