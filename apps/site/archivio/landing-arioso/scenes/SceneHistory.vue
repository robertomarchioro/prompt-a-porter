<template>
  <div class="feat">
    <div class="fc-copy">
      <div class="k">FEATURE · CRONOLOGIA</div>
      <h3 class="serif">Ogni modifica è una <em>versione</em>.</h3>
      <p>Ogni salvataggio crea uno snapshot. Confronta due versioni con un diff in stile git, e torna indietro quando l'ultima modifica ha rovinato il prompt che funzionava.</p>
      <div class="row2">
        <div class="s"><div class="v serif">v12</div><div class="l">snapshot su questo prompt</div></div>
        <div class="s"><div class="v serif">1 click</div><div class="l">ripristina qualsiasi versione</div></div>
      </div>
    </div>
    <div class="vercard">
      <div class="vh"><span class="t">Cronologia — Riassumi articolo</span><div class="chips"><span class="on">v7</span><span>v6</span><span>diff</span></div></div>
      <div class="vbody">
        <div class="tl">
          <div class="v cur"><span class="d"></span>v7 · attuale<span class="vt">oggi</span></div>
          <div class="v"><span class="d"></span>v6<span class="vt">3g</span></div>
          <div class="v"><span class="d"></span>v5<span class="vt">1M</span></div>
          <div class="v"><span class="d"></span>v4<span class="vt">1M</span></div>
          <div class="v"><span class="d"></span>v3<span class="vt">2M</span></div>
          <div class="v"><span class="d"></span>v2<span class="vt">2M</span></div>
          <div class="v"><span class="d"></span>v1<span class="vt">3M</span></div>
        </div>
        <div class="diff" v-pre>
          <div class="ln ctx"><span class="s"></span>Riassumi il seguente testo.</div>
          <div class="ln del"><span class="s">-</span>- Massimo 5 punti.</div>
          <div class="ln add"><span class="s">+</span>- Esattamente <span class="tok">{{numero_punti}}</span> punti (bullet).</div>
          <div class="ln add"><span class="s">+</span>- Ordine per importanza decrescente.</div>
          <div class="ln ctx"><span class="s"></span>Per ogni punto, una frase.</div>
          <div class="ln del"><span class="s">-</span>Tono: neutro.</div>
          <div class="ln add"><span class="s">+</span>Tono: <span class="tok">{{tono}}</span>.</div>
          <div class="ln ctx"><span class="s"></span>Chiudi con un TL;DR.</div>
          <div class="ln ctx"><span class="s"></span>Testo: <span class="tok">{{articolo}}</span></div>
        </div>
      </div>
    </div>
  </div>
</template>
