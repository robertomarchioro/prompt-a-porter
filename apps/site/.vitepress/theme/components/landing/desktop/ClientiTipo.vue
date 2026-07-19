<template>
  <div class="sez">
    <div class="label">A chi sta bene Prompt a Porter</div>
    <div class="cartellini">
      <div class="tag r1">
        <span class="foro" aria-hidden="true"></span>
        <div class="chi">Lo sviluppatore</div>
        <p>
          Trenta chiamate al giorno per review e refactor. Template versionabili come il codice —
          richiamati con <span class="cmd">pap render</span> o via MCP.
        </p>
      </div>
      <div class="tag r2">
        <span class="foro" aria-hidden="true"></span>
        <div class="chi">Il copywriter</div>
        <p>
          Cinquanta <span class="fileaccio">email_v3_FINAL</span>. Un brief, mille varianti — e il
          Ritocco quando un prompt è fiacco.
        </p>
      </div>
      <div class="tag r3">
        <span class="foro" aria-hidden="true"></span>
        <div class="chi">Il ricercatore</div>
        <p>Tre versioni dello stesso prompt a confronto. Quale rende meglio? Coi golden test, non a occhio.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sez {
  max-width: 1080px;
  margin: 0 auto;
  padding: 20px 48px 60px;
}
.label {
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--pap-faint);
  text-align: center;
  margin-bottom: 24px;
}
.cartellini {
  display: flex;
  gap: 20px;
}
.tag {
  flex: 1;
  position: relative;
  background: var(--pap-surface);
  border: 1px solid var(--pap-line);
  border-radius: 4px 4px 9px 9px;
  padding: 26px 22px 22px;
}
.tag.r1 {
  transform: rotate(-1deg);
}
.tag.r2 {
  transform: rotate(0.6deg);
}
.tag.r3 {
  transform: rotate(-0.6deg);
}
.foro {
  position: absolute;
  top: 9px;
  left: 50%;
  transform: translateX(-50%);
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 1.5px solid #c6c3b9;
  background: var(--pap-page);
}
.chi {
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--pap-viola);
  text-align: center;
}
.tag p {
  font-size: 14px;
  line-height: 1.55;
  color: var(--pap-sub);
  margin: 12px 0 0;
  text-align: center;
}
.cmd {
  font-family: var(--pap-mono);
  font-size: 12px;
  color: var(--pap-viola);
}
.fileaccio {
  font-family: var(--pap-mono);
  font-size: 12px;
  color: var(--pap-ambra-testo);
}
</style>
