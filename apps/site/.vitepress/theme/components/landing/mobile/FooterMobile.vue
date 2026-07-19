<template>
  <footer class="footer">
    <svg width="46" height="46" viewBox="0 0 46 46" class="rocchetto" aria-hidden="true">
      <rect x="10" y="6" width="26" height="34" rx="3" fill="#edeae1" stroke="#d8d6cc" stroke-width="1" />
      <rect x="6" y="6" width="34" height="5" rx="2" fill="#c7c4ba" />
      <rect x="6" y="35" width="34" height="5" rx="2" fill="#c7c4ba" />
      <g stroke="#8470d5" stroke-width="2">
        <line x1="12" y1="13" x2="34" y2="15" />
        <line x1="12" y1="17" x2="34" y2="19" />
        <line x1="12" y1="21" x2="34" y2="23" />
        <line x1="12" y1="25" x2="34" y2="27" />
        <line x1="12" y1="29" x2="34" y2="31" />
      </g>
      <path d="M34 15 C 42 18, 40 26, 44 30" fill="none" stroke="#8470d5" stroke-width="1.4" stroke-linecap="round" />
    </svg>
    <div class="barcode" aria-hidden="true"></div>
    <div class="care">
      100% locale · no cloud · nessun account · AGPL 3.0<br />
      <span class="firma">Disegnato da Roberto · cucito da <span class="claude">✦ Claude</span></span>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  padding: 22px 20px 26px;
  text-align: center;
}
.rocchetto {
  margin: 0 auto 12px;
  display: block;
}
.barcode {
  height: 32px;
  background: repeating-linear-gradient(
    90deg,
    var(--pap-ink) 0 2px,
    transparent 2px 4px,
    var(--pap-ink) 4px 5px,
    transparent 5px 9px,
    var(--pap-ink) 9px 12px,
    transparent 12px 14px
  );
  opacity: 0.8;
}
.care {
  font-family: var(--pap-mono);
  font-size: 11px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--pap-muted);
  margin-top: 10px;
  line-height: 1.8;
}
.firma {
  color: var(--pap-faint);
  text-transform: none;
  letter-spacing: 0.06em;
}
.claude {
  color: #d97757;
}
</style>
