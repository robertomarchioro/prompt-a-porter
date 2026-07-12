<script setup lang="ts">
import { withBase } from 'vitepress'
import { AUTHOR, DOCS, GITHUB, ISSUES, LICENSE, RELEASES } from './links'
import { useNomeOsVisitatore } from './os'
import { useLinkDownload } from './download'

const osNome = useNomeOsVisitatore()
const linkDownload = useLinkDownload()
</script>

<template>
  <footer class="footer">
    <div class="ft-in">
      <div class="ft-top">
        <div class="ft-brand">
          <div class="ft-mark"><img class="mk" :src="withBase('/icons/icon-64.png')" alt="" width="30" height="30" /><span class="nm">Prompt-à-porter</span></div>
          <p class="ft-tag">La tua collezione di prompt AI. <em>Pronti da indossare.</em></p>
          <div class="ft-cta"><a class="p" :href="linkDownload">{{ osNome ? `Scarica per ${osNome}` : "Scarica l'app" }}</a><a class="g" :href="GITHUB">Vedi su GitHub</a></div>
        </div>
        <nav class="ft-col">
          <h4>La collezione</h4>
          <a href="#come-funziona">Versioni &amp; diff</a>
          <a href="#come-funziona">Segnaposti</a>
          <a href="#come-funziona">Ricerca semantica</a>
          <a href="#come-funziona">Varianti A/B</a>
          <a href="#come-funziona">Import composti</a>
        </nav>
        <nav class="ft-col">
          <h4>Atelier</h4>
          <a href="#come-funziona">Come funziona</a>
          <a href="#come-funziona">La palette ⌃⇧P</a>
          <a href="#collezione">Local-first</a>
          <a :href="RELEASES">Changelog</a>
        </nav>
        <nav class="ft-col">
          <h4>La casa</h4>
          <a :href="GITHUB">GitHub</a>
          <a :href="DOCS">Documentazione</a>
          <a :href="LICENSE">Licenza AGPL 3.0</a>
          <a :href="ISSUES">Segnala un problema</a>
        </nav>
      </div>

      <div class="ft-label">
        <span class="stitch">Etichetta di manutenzione</span>
        <div class="care">
          <span>100% locale</span>
          <span>no cloud</span>
          <span>nessun account</span>
          <span>lavabile in git</span>
          <span>AGPL 3.0</span>
        </div>
      </div>

      <div class="ft-bottom">
        <span class="cp">© 2026 Prompt-à-porter · Edizione locale · Stagione AW26</span>
        <div class="ft-sign">
          <span class="by">Disegnato da</span><a class="name" :href="AUTHOR">Roberto</a>
          <span class="by">cucito da</span>
          <!-- Glyph sparkle generico, NON il marchio Anthropic (cfr. handoff) -->
          <span class="name claude"><svg class="spark" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 0c1 7 4 10 12 12-8 2-11 5-12 12-1-7-4-10-12-12 8-2 11-5 12-12Z" fill="currentColor" /></svg>Claude</span>
        </div>
      </div>
    </div>
  </footer>
</template>
