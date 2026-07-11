<template>
  <div class="feat">
    <div class="fc-copy">
      <div class="k">FEATURE · IMPORT</div>
      <h3 class="serif">Un blocco, <em>dieci capi</em>.</h3>
      <p v-pre>Definisci una volta il ruolo o lo stile, poi cucilo dove vuoi con <code>{{import "ruolo-engineer"}}</code>. Cambi il blocco sorgente e tutti i prompt che lo indossano si aggiornano.</p>
      <div class="row2">
        <div class="s"><div class="v serif">1 → ∞</div><div class="l">una sorgente, molti prompt</div></div>
        <div class="s"><div class="v serif">0</div><div class="l">duplicati da mantenere</div></div>
      </div>
    </div>
    <div class="impcard">
      <div class="imp-h"><span class="t">Code review strutturata</span><span class="badge">composto</span></div>
      <div class="imp-sources">
        <div class="src-chip"><span class="ic">⊕</span> ruolo-engineer<span class="v">v4</span></div>
        <div class="src-chip"><span class="ic">⊕</span> stile-conciso<span class="v">v2</span></div>
        <div class="src-chip"><span class="ic">⊕</span> formato-md<span class="v">v1</span></div>
      </div>
      <div class="imp-arrow">cuce in ↓</div>
      <div class="imp-body" v-pre>
        <div class="il imported">{{import "ruolo-engineer"}}</div>
        <div class="il imported">{{import "stile-conciso"}}</div>
        <div class="il imported">{{import "formato-md"}}</div>
        <div class="il blank"></div>
        <div class="il">Analizza il diff per <span class="tok">{{severità}}</span> e cita</div>
        <div class="il">sempre le righe interessate. Raggruppa</div>
        <div class="il">i rilievi per file e per gravità.</div>
      </div>
      <div class="imp-foot"><span>IMPORT COMPOSTI</span><span class="n">2</span></div>
    </div>
  </div>
</template>
