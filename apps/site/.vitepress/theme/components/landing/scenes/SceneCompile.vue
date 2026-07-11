<template>
  <div class="feat">
    <div class="fc-copy">
      <div class="k">FEATURE · SEGNAPOSTI</div>
      <h3 class="serif">Compila e <em>incolla</em>, senza copia-incolla.</h3>
      <p v-pre>Il prompt è un capo, i <code>{{segnaposti}}</code> sono le misure. Riempi i campi, premi <code>⌃↵</code> e il testo compilato finisce dritto nel campo in cui stavi scrivendo.</p>
      <div class="row2">
        <div class="s"><div class="v serif">0 → 4</div><div class="l">campi compilati al volo</div></div>
        <div class="s"><div class="v serif">1 tasto</div><div class="l">compila e incolla</div></div>
      </div>
    </div>
    <div class="segcard">
      <div class="sh"><span class="bk">←</span> Brainstorm con criteri</div>
      <div class="sub2"><span class="l">SEGNAPOSTI</span><span class="c">2 di 4 compilati</span></div>
      <div class="segrow filled"><span class="lbl">obiettivo</span><div class="inp">nomi per il prodotto</div></div>
      <div class="segrow filled"><span class="lbl">contesto</span><div class="inp">app desktop, target dev</div></div>
      <div class="segrow"><span class="lbl">numero</span><div class="inp ph">numero</div></div>
      <div class="segrow"><span class="lbl">criteri</span><div class="inp ph">criteri</div></div>
      <div class="prev">
        <div class="l">ANTEPRIMA</div>
        <div class="box" v-pre>Genera idee per: <span class="fill">nomi per il prodotto</span>.<br />Contesto: <span class="fill">app desktop, target dev</span><br />Vincoli: esattamente <span class="tok">{{numero}}</span> idee, valuta su <span class="tok">{{criteri}}</span>.</div>
      </div>
      <div class="cf"><span class="ci"><kbd>esc</kbd> indietro</span><span class="ci"><kbd>⌃</kbd><kbd>↵</kbd> compila e incolla</span></div>
    </div>
  </div>
</template>
