<script setup lang="ts">
import { withBase } from 'vitepress'
import {
  GITHUB,
  DOCS,
  GUIDA,
  LICENSE,
  ISSUES,
  CHANGELOG,
  DOC_SINTASSI,
  DOC_RICERCA,
  DOC_VARIANTI,
  DOC_IMPORT,
  DOC_SCORCIATOIE,
} from '../links'
</script>

<template>
  <footer class="footer">
    <div class="griglia">
      <div>
        <div class="brand">
          <img class="mark" :src="withBase('/icons/icon-64.png')" alt="" width="30" height="30" />
          <span class="nome">Prompt-à-porter</span>
        </div>
        <p class="tagline">La tua collezione di prompt AI. <em>Pronti da indossare.</em></p>
      </div>
      <div>
        <div class="col-label">La collezione</div>
        <a :href="DOC_SINTASSI">Segnaposti</a>
        <a :href="DOC_RICERCA">Ricerca semantica</a>
        <a :href="DOC_VARIANTI">Varianti A/B</a>
        <a :href="DOC_IMPORT">Import composti</a>
      </div>
      <div>
        <div class="col-label">Atelier</div>
        <a href="#come-funziona">Come funziona</a>
        <a :href="GUIDA">Getting started</a>
        <a :href="DOC_SCORCIATOIE">La palette ⌃⇧P</a>
        <a :href="CHANGELOG">Changelog</a>
      </div>
      <div>
        <div class="col-label">La casa</div>
        <a :href="GITHUB">GitHub</a>
        <a :href="DOCS">Documentazione</a>
        <a :href="LICENSE">Licenza AGPL 3.0</a>
        <a :href="ISSUES">Segnala un problema</a>
      </div>
    </div>
    <div class="fondo">
      <div class="riga">
        <div class="sarto">
          <svg width="38" height="38" viewBox="0 0 46 46" class="rocchetto" aria-hidden="true">
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
        </div>
        <div class="firma">
          <div class="copy">© 2026 Prompt-à-porter · Edizione locale · Stagione AW26</div>
          <div class="cucito">
            Disegnato da <span class="roberto">Roberto</span> · cucito da <span class="claude">✦ Claude</span>
          </div>
        </div>
      </div>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  background: var(--pap-surface);
  border-top: 1px solid var(--pap-line);
}
.griglia {
  max-width: 1080px;
  margin: 0 auto;
  padding: 56px 48px 30px;
  display: grid;
  grid-template-columns: 1.4fr 1fr 1fr 1fr;
  gap: 44px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 11px;
}
.mark {
  width: 30px;
  height: 30px;
  display: block;
}
.nome {
  font-family: var(--pap-serif);
  font-size: 21px;
}
.tagline {
  font-family: var(--pap-serif);
  font-weight: 400;
  font-size: 19px;
  line-height: 1.4;
  color: var(--pap-sub);
  margin: 18px 0 0;
  max-width: 280px;
}
.tagline em {
  font-style: italic;
  color: var(--pap-viola);
}
.col-label {
  font-family: var(--pap-mono);
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--pap-faint);
  margin-bottom: 16px;
}
.griglia a {
  display: block;
  font-size: 13.5px;
  color: var(--pap-sub);
  padding: 6px 0;
}
.griglia a:hover {
  color: var(--pap-viola);
}
.fondo {
  max-width: 1080px;
  margin: 0 auto;
  padding: 0 48px 40px;
}
.riga {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 22px 0 0;
  border-top: 1px solid var(--pap-line);
}
.sarto {
  display: flex;
  align-items: center;
  gap: 16px;
}
.rocchetto {
  flex: none;
}
.barcode {
  height: 34px;
  width: 220px;
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
.firma {
  text-align: right;
}
.copy {
  font-size: 12px;
  color: var(--pap-muted);
}
.cucito {
  font-family: var(--pap-mono);
  font-size: 10.5px;
  letter-spacing: 0.06em;
  color: var(--pap-faint);
  margin-top: 4px;
}
.roberto {
  font-family: var(--pap-serif);
  font-style: italic;
  font-size: 15px;
  color: var(--pap-sub);
}
.claude {
  color: #d97757;
}

@media (max-width: 1080px) {
  .griglia {
    padding: 48px 36px 26px;
  }
  .fondo {
    padding: 0 36px 36px;
  }
}

@media (max-width: 900px) {
  .griglia {
    grid-template-columns: 1fr 1fr;
    gap: 32px;
  }
}
</style>
