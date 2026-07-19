<template>
  <div class="banda">
    <div class="alone" aria-hidden="true"></div>
    <div class="in">
      <div class="kicker">— manifesto —</div>
      <div class="slogan">
        Se ci puoi scrivere,<br />ci puoi <em>incollare il tuo prompt.</em>
      </div>
      <div class="tecnica">in ogni editor, applicazione o sito web.</div>
    </div>
  </div>
</template>

<style scoped>
.banda {
  margin: 16px 0 0;
  padding: 30px 24px;
  background: #edebf6;
  border-top: 1px solid var(--pap-line);
  border-bottom: 1px solid var(--pap-line);
  text-align: center;
  position: relative;
  overflow: hidden;
}
.alone {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse 70% 120% at 50% 50%, rgba(132, 112, 213, 0.12), transparent 70%);
  pointer-events: none;
}
.in {
  position: relative;
}
.kicker {
  font-family: var(--pap-mono);
  font-size: 12.5px;
  letter-spacing: 0.28em;
  text-transform: uppercase;
  color: var(--pap-viola);
  margin-bottom: 14px;
}
.slogan {
  font-family: var(--pap-serif);
  font-weight: 400;
  font-size: 29px;
  line-height: 1.06;
  letter-spacing: -0.02em;
  color: var(--pap-ink);
}
.slogan em {
  font-style: italic;
  color: var(--pap-viola);
}
.tecnica {
  font-family: var(--pap-mono);
  font-size: 13px;
  letter-spacing: 0.04em;
  color: var(--pap-muted);
  margin-top: 16px;
}
</style>
