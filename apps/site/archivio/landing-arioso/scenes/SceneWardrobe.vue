<script setup lang="ts">
const ROWS = ['Riassumi articolo', 'Commit message', 'Code review', 'Email cliente', 'Brainstorm idee']
</script>

<template>
  <div class="feat">
    <div class="fc-copy">
      <div class="k">FEATURE · GUARDAROBA</div>
      <h3 class="serif">Abito da giorno,<br /><em>abito da sera</em>.</h3>
      <p>Tema chiaro per le ore di luce, tema scuro per le sessioni notturne. La stessa collezione, due mise — si cambia con un tocco.</p>
      <div class="row2">
        <div class="s"><div class="v serif">2 mise</div><div class="l">chiaro e scuro</div></div>
        <div class="s"><div class="v serif">1 tocco</div><div class="l">per cambiare</div></div>
      </div>
    </div>
    <div class="wardrobe">
      <div class="mini day">
        <div class="m-tag">Abito da giorno</div>
        <div class="m-ui">
          <div class="m-search"><span class="mag">⌕</span> Cerca prompt…</div>
          <div v-for="(row, k) in ROWS" :key="row" class="m-row" :class="{ on: k === 0 }"><span v-text="row"></span><span class="mb">privato</span></div>
        </div>
      </div>
      <div class="toggle"><span class="sun">☀</span><span class="moon">☾</span><span class="knob"></span></div>
      <div class="mini night">
        <div class="m-tag">Abito da sera</div>
        <div class="m-ui">
          <div class="m-search"><span class="mag">⌕</span> Cerca prompt…</div>
          <div v-for="(row, k) in ROWS" :key="row" class="m-row" :class="{ on: k === 0 }"><span v-text="row"></span><span class="mb">privato</span></div>
        </div>
      </div>
    </div>
  </div>
</template>
