<template>
  <div class="sez">
    <div class="label">A chi sta bene</div>
    <div class="cartellini">
      <div class="tag r1">
        <span class="foro" aria-hidden="true"></span>
        <div class="chi">Sviluppatore</div>
        <p>Trenta chiamate al giorno. Template versionabili come il codice.</p>
      </div>
      <div class="tag r2">
        <span class="foro" aria-hidden="true"></span>
        <div class="chi">Copywriter</div>
        <p>Cinquanta <span class="fileaccio">email_FINAL</span>. Un brief, mille varianti.</p>
      </div>
      <div class="tag r3">
        <span class="foro" aria-hidden="true"></span>
        <div class="chi">Ricercatore</div>
        <p>Tre versioni a confronto. Coi golden test, non a occhio.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sez {
  padding: 24px 16px 2px;
}
.label {
  font-family: var(--pap-mono);
  font-size: 12.5px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--pap-faint);
  text-align: center;
  margin-bottom: 16px;
}
.cartellini {
  display: flex;
  gap: 9px;
  align-items: flex-start;
}
.tag {
  flex: 1;
  position: relative;
  background: var(--pap-surface);
  border: 1px solid var(--pap-line);
  border-radius: 4px 4px 7px 7px;
  padding: 17px 11px 13px;
}
.tag.r1 {
  transform: rotate(-1.5deg);
}
.tag.r2 {
  transform: rotate(1deg);
}
.tag.r3 {
  transform: rotate(-1deg);
}
.foro {
  position: absolute;
  top: 6px;
  left: 50%;
  transform: translateX(-50%);
  width: 8px;
  height: 8px;
  border-radius: 50%;
  border: 1.5px solid #c6c3b9;
  background: var(--pap-page);
}
.chi {
  font-family: var(--pap-mono);
  font-size: 12px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--pap-viola);
  text-align: center;
  margin-top: 4px;
}
.tag p {
  font-size: 13.5px;
  line-height: 1.45;
  color: var(--pap-sub);
  margin: 8px 0 0;
  text-align: center;
}
.fileaccio {
  font-family: var(--pap-mono);
  font-size: 12.5px;
  color: var(--pap-ambra-testo);
}
</style>
