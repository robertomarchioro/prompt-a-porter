<template>
  <section class="wrap sec tight" id="collezione">
    <div class="colhead"><h2 class="serif">La collezione</h2><span class="label">Cinque capi · una stagione</span></div>
    <div class="grid">
      <article class="card big cc-violet">
        <div class="swatch"><span class="hang"></span><span class="sku">VER</span></div>
        <div class="body"><h3 class="serif">Ogni modifica, in vetrina</h3><p>Ogni save crea una versione. Diff stile git. Torna indietro quando l'ultima modifica ha rovinato tutto.</p></div>
      </article>
      <article class="card big cc-amber">
        <div class="swatch"><span class="hang"></span><span class="sku">TPL</span></div>
        <div class="body"><h3 class="serif">Capi su misura</h3><p v-pre><code>{{nome}}</code> per variabili ad-hoc, <code>{{globale autore}}</code> dal vault. Il prompt diventa template, il template diventa abito su misura.</p></div>
      </article>
      <article class="card sm cc-rose">
        <div class="swatch"><span class="hang"></span><span class="sku">SRCH</span></div>
        <div class="body"><h3 class="serif">Cerca al buio dell'armadio</h3><p>Ricerca per significato, non per parola. Modello ONNX MiniLM embedded — niente cloud, niente quote.</p></div>
      </article>
      <article class="card sm cc-sage">
        <div class="swatch"><span class="hang"></span><span class="sku">A/B</span></div>
        <div class="body"><h3 class="serif">Prova prima di comprare</h3><p>Varianti A/B dello stesso prompt, confrontate fianco a fianco. Scegli con i golden test.</p></div>
      </article>
      <article class="card sm cc-blue">
        <div class="swatch"><span class="hang"></span><span class="sku">IMP</span></div>
        <div class="body"><h3 class="serif">Un tono, dieci tagli</h3><p v-pre><code>{{import "ruolo"}}</code> per riusare blocchi. Definisci lo stile una volta, riusalo ovunque. Sartoria modulare.</p></div>
      </article>
    </div>
  </section>
</template>
