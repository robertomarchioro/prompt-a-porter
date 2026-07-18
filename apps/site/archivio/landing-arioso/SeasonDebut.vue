<script setup lang="ts">
import { useNomeOsVisitatore } from './os'
import { useLinkDownload } from './download'

const osNome = useNomeOsVisitatore()
const linkDownload = useLinkDownload()
</script>

<template>
  <section class="season" id="stagione">
    <div class="sin">
      <div class="s-copy">
        <div class="plate">Autunno-Inverno 2026 · Debutto</div>
        <h2 class="serif">Arioso <em>Atelier</em></h2>
        <div class="verline">
          <span class="v">v<b>1.0</b></span>
          <span class="from">subentra a <em>«Ago e Filo»</em>, il codename di laboratorio</span>
        </div>
        <p class="lede">Il capo era ancora <em>in cucitura</em>. Con la 1.0 l'ago si posa: Prompt-à-porter smette di essere in beta e apre la sua prima stagione in negozio.</p>
        <div class="scta">
          <a class="b1" :href="linkDownload">{{ osNome ? `Scarica la 1.0 per ${osNome}` : 'Scarica la 1.0' }}</a>
          <span class="note">gratis · local-first · AGPL 3.0</span>
        </div>
      </div>
      <div class="card-launch">
        <div class="cl-top">
          <span class="tag">Nella collezione di debutto</span>
          <span class="stamp">v1.0 · AI 2026</span>
        </div>
        <div class="cl-list">
          <div class="cl-row"><span class="ic">↓</span><div class="tx"><div class="t">Installazione senza attriti</div><div class="d">Setup rapido per Windows, macOS e Linux; aggiornamenti automatici e silenziosi in background.</div></div></div>
          <div class="cl-row"><span class="ic">✒</span><div class="tx"><div class="t">Editor rifinito su misura</div><div class="d">Vista Sorgente e Compilato affiancate, varianti create al volo, import e export Markdown con front-matter.</div></div></div>
          <div class="cl-row"><span class="ic">⊕</span><div class="tx"><div class="t">Prompt componibili evoluti</div><div class="d" v-pre><code>{{import "x" with k=v}}</code> con variabili scopate e pinning a una versione storica.</div></div></div>
          <div class="cl-row"><span class="ic">◉</span><div class="tx"><div class="t">Solida da indossare tutti i giorni</div><div class="d">Accessibilità di base, suite di regressione verde e documentazione utente completa.</div></div></div>
        </div>
        <div class="cl-foot"><span class="dot"></span>Penelope ha finito la tela — la prima release stabile della linea Personale.</div>
      </div>
    </div>
  </section>
</template>
