<template>
  <div class="hero">
    <span class="eyebrow">
      <span class="kbd">⌃⇧P</span>
      <span class="etesto">in qualsiasi app</span>
    </span>
    <h1>Prompt-<em>à</em>-porter</h1>
    <p class="sub">La tua collezione di prompt AI.<br /><em>Pronti da indossare.</em></p>
    <p class="pitch">
      Un tasto magico e il prompt giusto è tra le dita: lo compili e lo incolli dove stai
      scrivendo.
    </p>
  </div>
</template>

<style scoped>
.hero {
  padding: 24px 24px 20px;
  text-align: center;
}
.eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: var(--pap-surface);
  border: 1px solid var(--pap-line);
  border-radius: 100px;
  padding: 6px 14px 6px 8px;
  margin-bottom: 18px;
}
.kbd {
  font-family: var(--pap-mono);
  font-size: 10px;
  background: var(--pap-white);
  border: 1px solid var(--pap-line);
  border-bottom-width: 2px;
  border-radius: 4px;
  padding: 2px 6px;
  color: var(--pap-viola);
}
.etesto {
  font-size: 10.5px;
  color: #6e6c60;
}
h1 {
  font-family: var(--pap-serif);
  font-weight: 300;
  font-size: 54px;
  line-height: 0.9;
  letter-spacing: -0.04em;
  margin: 0;
}
h1 em {
  font-style: italic;
  color: var(--pap-viola);
}
.sub {
  font-family: var(--pap-serif);
  font-weight: 300;
  font-size: 19px;
  line-height: 1.4;
  color: var(--pap-sub);
  margin: 20px 6px 0;
}
.sub em {
  font-style: italic;
  color: var(--pap-ink);
}
.pitch {
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--pap-muted);
  margin: 14px 8px 0;
}
</style>
