<template>
  <!--
    Mockup HTML/CSS della finestra app (riferimento visivo, cfr. handoff).
    In produzione va sostituito con uno screenshot/registrazione reale dell'app.
    v-pre: i token {{…}} sono testo letterale, non interpolazioni Vue.
  -->
  <div class="appwin only-desktop">
    <div class="a-title">
      <div class="l"><span class="pp serif">P</span><span class="tt">Prompt a Porter</span><span class="ver">v1.0 Arioso Atelier</span></div>
      <div class="r"><span>◐</span><span>?</span><span>⚙</span></div>
    </div>
    <div class="a-body">
      <div class="a-col sb">
        <div class="ws"><span class="av">P</span><span class="nm">Personale</span><span class="ch">▾</span></div>
        <div class="grp">VISTE</div>
        <div class="it on"><span class="ic">◷</span>Recenti</div>
        <div class="it"><span class="ic">★</span>Preferiti<span class="ct">3</span></div>
        <div class="it"><span class="ic">▦</span>Tutti i prompt<span class="ct">17</span></div>
        <div class="it"><span class="ic">🗑</span>Cestino<span class="ct">0</span></div>
        <div class="grp">VISIBILITÀ</div>
        <div class="it"><span class="dotp"></span>Privati<span class="ct">15</span></div>
        <div class="it"><span class="dotp"></span>Team<span class="ct">2</span></div>
        <div class="grp">CARTELLE <span class="fr">7</span></div>
        <div class="grp">TAG <span class="fr">8</span></div>
        <div class="foot">
          <div class="it"><span class="ic">▤</span>Insight</div>
          <div class="it"><span class="ic">⚠</span>Regressioni</div>
        </div>
      </div>
      <div class="a-col lst">
        <div class="hd"><span class="ti">Recenti</span><span class="cnt">17</span></div>
        <div class="sr">⌕ Cerca prompt…</div>
        <div class="new">+ Nuovo</div>
        <div class="row on"><span class="dt"></span><div class="tx"><div class="nm">Riassumi articolo in N punti</div></div><span class="tm">1M fa</span></div>
        <div class="row"><span class="dt"></span><div class="tx"><div class="nm">Commit message da diff</div></div><span class="tm">1M fa</span></div>
        <div class="row"><span class="dt"></span><div class="tx"><div class="nm">Email professionale parametrica</div></div><span class="st">★</span></div>
        <div class="row"><span class="dt"></span><div class="tx"><div class="nm">Code review strutturata</div></div><span class="st">★</span></div>
        <div class="row"><span class="dt"></span><div class="tx"><div class="nm">Traduzione tecnica con glossario</div></div><span class="tm">1M fa</span></div>
      </div>
      <div class="a-col ed">
        <div class="eh">
          <div class="ti">
            <h3 class="serif">Riassumi articolo in N punti</h3>
            <div class="acts"><span>★</span><span>⑂</span><span>↓</span><span class="comp">Compila</span></div>
          </div>
          <div class="ds">Distilla un testo lungo in N punti chiave + TL;DR + verdetto.</div>
          <div class="meta">
            <span class="pill"><span class="g">●</span> Privato</span>
            <span class="pill"><span class="f">▣</span> Produttività</span>
            <span class="pill">gpt-4-mini</span>
            <span class="x">Usato 31×</span><span class="x">· Aggiornato 37g fa</span>
          </div>
        </div>
        <div class="tabs"><span class="on">Editor</span><span>Anteprima</span><span>Diagnosi</span><span>Test golden</span><span>Cronologia</span><span>Import &amp; Var.</span></div>
        <div class="toolbar"><span>B</span><span><i>I</i></span><span>H1</span><span>H2</span><span>≔</span><span>❝</span><span>&lt;/&gt;</span><span>🔗</span></div>
        <div class="code">
          <div class="gut">1<br />2<br />3<br />4<br />5<br />6<br />7</div>
          <div class="src" v-pre>Riassumi il seguente testo in italiano.<br /><br /><span class="b">Vincoli:</span><br />- Esattamente <span class="tok">{{numero_punti}}</span> punti chiave (bullet).<br />- Ordine per importanza decrescente.<br /><br />Testo: <span class="tok">{{articolo}}</span></div>
        </div>
        <div class="sbar"><span class="dot"></span>salvato<span>L:10 C:1</span><span>267 char</span><span>~67 tok</span></div>
      </div>
      <div class="a-col mt">
        <div class="grp"><div class="h">METADATI</div></div>
        <div class="grp"><div class="h">VISIBILITÀ</div><div class="fld">Privato <span class="ch">▾</span></div></div>
        <div class="grp"><div class="h">MODELLO TARGET</div><div class="fld">gpt-4-mini</div></div>
        <div class="grp"><div class="h">TAG</div><span class="tag">● produttività</span></div>
        <div class="grp">
          <div class="h">SEGNAPOSTI RILEVATI <span class="n">2</span></div>
          <div class="seg" v-pre>{{numero_punti}}<span class="ty">testo</span></div>
          <div class="seg" v-pre>{{articolo}}<span class="ty">testo</span></div>
        </div>
        <div class="grp"><div class="hb"><div class="h">VARIANTI A/B</div><span class="ct">0</span></div></div>
      </div>
    </div>
    <div class="a-status">
      <div class="l"><span class="dot"></span>vault locale<span>· Riassumi articolo in N punti</span></div>
      <div class="r"><span>salvato 2m fa</span><span class="kb">Ctrl+Shift+P</span><span>cerca</span></div>
    </div>
  </div>

  <!-- Variante mobile (≤900px, handoff UX Fix 1): la finestra 4-colonne
       non si scala — al suo posto una card verticale con l'essenza della
       libreria, testo ≥ 13px e righe da 44px. -->
  <div class="m-lib only-mobile">
    <div class="mh">La tua libreria</div>
    <div class="msr">⌕ Cerca prompt, tag o azione…</div>
    <div class="mrow on"><div><div class="mnm">Riassumi articolo in N punti</div><div class="mmeta">privato · v7 · usato 31×</div></div><span class="mstar">★</span></div>
    <div class="mrow"><div><div class="mnm">Commit message da diff</div><div class="mmeta">privato · v4</div></div></div>
    <div class="mrow"><div><div class="mnm">Email professionale parametrica</div><div class="mmeta">privato · v9</div></div><span class="mstar">★</span></div>
    <div class="mrow"><div><div class="mnm">Code review strutturata</div><div class="mmeta">composto · v6</div></div></div>
    <div class="mfoot"><span class="dot"></span>17 prompt · vault locale, niente cloud</div>
  </div>
</template>
