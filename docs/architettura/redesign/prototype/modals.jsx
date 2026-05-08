// =====================================================
// Modals: Compila, Cronologia, Insight, Regressioni, Impostazioni, Palette
// =====================================================
const { useState: useStateM, useEffect: useEffectM, useMemo: useMemoM, useRef: useRefM } = React;

function Modal({ open, onClose, title, sub, width, children, footer }) {
  if (!open) return null;
  return (
    <div className="modal-scrim" onClick={onClose}>
      <div className="modal" style={{ ['--modal-w']: width || '720px' }} onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <div className="modal-title">{title}</div>
          {sub && <div className="modal-sub">{sub}</div>}
          <button className="icon-btn modal-close" onClick={onClose}><Icon name="x" size={14} /></button>
        </div>
        <div className="modal-body">{children}</div>
        {footer && <div className="modal-footer">{footer}</div>}
      </div>
    </div>
  );
}

// ---------- Compila ----------
function CompilaModal({ open, onClose, prompt }) {
  const [vals, setVals] = useStateM({ specializzazione: 'backend FastAPI + asyncio', stile_docstring: 'google', contesto: '' });
  const [format, setFormat] = useStateM('Testo');
  const [expandImports, setExpandImports] = useStateM(true);
  if (!prompt) return null;

  const compiled = (prompt.body || '')
    .replace(/\{\{import\s+"([^"]+)"\}\}/g, (_, p) => expandImports ? `[+ ${p}: regole PEP8 strict importate]` : `{{import "${p}"}}`)
    .replace(/\{\{(\w+)\}\}/g, (_, k) => vals[k] || `{{${k}}}`);

  const placeholders = prompt.placeholders || [];
  const filled = placeholders.filter(p => (vals[p.name] || '').length > 0).length;

  return (
    <Modal open={open} onClose={onClose} title="Compila" sub={prompt.title} width="900px"
      footer={
        <React.Fragment>
          <span style={{ fontSize: 12, color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
            <span className="status-dot ok" style={{ display: 'inline-block', marginRight: 6, verticalAlign: 'middle' }}></span>
            {filled === placeholders.length ? 'Pronto' : `${filled}/${placeholders.length} compilati`} · ~{Math.round(compiled.length / 4)} token
          </span>
          <span style={{ flex: 1 }}></span>
          <button className="btn-ghost" onClick={onClose}>Annulla</button>
          <button className="btn-primary"><Icon name="copy" size={12} /> Compila e copia <kbd style={{marginLeft:6, color:'rgba(0,0,0,0.4)'}}>⌃↵</kbd></button>
        </React.Fragment>
      }>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, height: 460 }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 10, overflow: 'auto' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
            <span style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Variabili</span>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--text-muted)' }}>{filled}/{placeholders.length}</span>
            <div style={{ flex: 1, height: 4, background: 'var(--bg-overlay)', borderRadius: 999, overflow: 'hidden' }}>
              <div style={{ width: `${(filled / Math.max(placeholders.length, 1)) * 100}%`, height: '100%', background: 'var(--accent-team)' }}></div>
            </div>
          </div>
          {placeholders.map(p => (
            <div className="field" key={p.name}>
              <label className="field-label">
                <span className="placeholder-token" style={{ display: 'inline-block', padding: '1px 6px' }}>{`{{${p.name}}}`}</span>
                {p.required && <span style={{ color: 'var(--danger)', marginLeft: 6 }}>*</span>}
                <span style={{ marginLeft: 8, color: 'var(--text-subtle)' }}>{p.type}</span>
              </label>
              {p.type === 'multilinea'
                ? <textarea className="field-input" style={{ minHeight: 60, padding: 8, fontFamily: 'var(--font-mono)' }} value={vals[p.name] || ''} onChange={e => setVals({ ...vals, [p.name]: e.target.value })} placeholder={p.default} />
                : <input className="field-input" style={{ fontFamily: 'var(--font-mono)' }} value={vals[p.name] || ''} onChange={e => setVals({ ...vals, [p.name]: e.target.value })} placeholder={p.default} />}
            </div>
          ))}
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 0', borderTop: '1px solid var(--border-subtle)', marginTop: 6 }}>
            <span className={`toggle ${expandImports ? 'on' : ''}`} onClick={() => setExpandImports(v => !v)}></span>
            <span style={{ fontSize: 12 }}>Espandi <code style={{fontFamily:'var(--font-mono)'}}>{`{{import}}`}</code> ricorsivamente</span>
          </div>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', minHeight: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 8 }}>
            <span style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Anteprima</span>
            <div className="segmented" style={{ marginLeft: 'auto' }}>
              {['Testo', 'Markdown', 'JSON'].map(f => (
                <button key={f} className={format === f ? 'on' : ''} onClick={() => setFormat(f)}>{f}</button>
              ))}
            </div>
          </div>
          <div style={{ flex: 1, background: 'var(--bg-input)', border: '1px solid var(--border-subtle)', borderRadius: 8, padding: 12, fontFamily: 'var(--font-mono)', fontSize: 12.5, lineHeight: 1.6, overflow: 'auto', whiteSpace: 'pre-wrap' }}>
            {compiled}
          </div>
        </div>
      </div>
    </Modal>
  );
}

// ---------- Cronologia ----------
function CronologiaModal({ open, onClose, prompt }) {
  const [activeV, setActiveV] = useStateM('v3');
  const [mode, setMode] = useStateM('side');
  if (!prompt) return null;
  const history = prompt.history || [];

  return (
    <Modal open={open} onClose={onClose} title="Cronologia versioni" sub={prompt.title} width="980px"
      footer={
        <React.Fragment>
          <span style={{ fontSize: 12, color: 'var(--text-muted)' }}>3 versioni · creato il 4 mag 2026</span>
          <span style={{ flex: 1 }}></span>
          <button className="btn-ghost" onClick={onClose}>Chiudi</button>
          <button className="btn-secondary"><Icon name="export" size={12} /> Esporta diff MD</button>
          <button className="btn-secondary"><Icon name="history" size={12} /> Ripristina questa versione</button>
        </React.Fragment>
      }>
      <div className="history-grid">
        <div className="history-list">
          {history.map(h => (
            <div key={h.v} className={`history-row ${activeV === h.v ? 'active' : ''}`} onClick={() => setActiveV(h.v)}>
              <div className="v">
                {h.v}
                {h.head && <span className="head-pill">TESTA</span>}
              </div>
              <div className="author">
                <span className={`av ${h.team ? 'team' : ''}`}>{h.authorInit}</span>
                <span>{h.author}</span>
                <span style={{ flex: 1 }}></span>
                <span className="ts">{h.ts.split(' ')[1]}</span>
              </div>
              <div className="summary">{h.summary}</div>
              <div className="delta">
                <span className="add">+{h.delta.add}</span>
                <span className="del">−{h.delta.del}</span>
              </div>
            </div>
          ))}
        </div>
        <div className="history-detail">
          <div className="history-detail-head">
            <div>
              <div style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Versione {activeV.replace('v','')}</div>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--text-strong)', marginTop: 2 }}>
                Modifiche di {history.find(h => h.v === activeV)?.author}
              </div>
              <div style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-muted)', marginTop: 2 }}>
                {history.find(h => h.v === activeV)?.ts}
              </div>
            </div>
            <div style={{ marginLeft: 'auto' }} className="segmented">
              <button className={mode === 'body' ? 'on' : ''} onClick={() => setMode('body')}>Body</button>
              <button className={mode === 'inline' ? 'on' : ''} onClick={() => setMode('inline')}>Diff inline</button>
              <button className={mode === 'side' ? 'on' : ''} onClick={() => setMode('side')}>Side-by-side</button>
            </div>
          </div>
          <div className="history-detail-body">
            {mode === 'side' ? (
              <div className="diff-side">
                <div className="diff-col before">
                  <div style={{ fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', marginBottom: 6 }}>v2 — precedente</div>
                  <div className="diff-line">Sei un esperto sviluppatore Python con 30 anni di esperienza.</div>
                  <div className="diff-line"> </div>
                  <div className="diff-line">Specializzato in: {`{{specializzazione}}`}</div>
                  <div className="diff-line"> </div>
                  <div className="diff-line">Quando rispondi:</div>
                  <div className="diff-line">1. Usa type hints completi (PEP 484)</div>
                  <div className="diff-line del">2. Preferisci dataclasses per validation</div>
                  <div className="diff-line">3. Scrivi docstring stile google</div>
                  <div className="diff-line">4. Suggerisci sempre test pytest</div>
                </div>
                <div className="diff-col">
                  <div style={{ fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', marginBottom: 6 }}>v3 — selezionata</div>
                  <div className="diff-line">Sei un esperto sviluppatore Python con 30 anni di esperienza.</div>
                  <div className="diff-line"> </div>
                  <div className="diff-line">Specializzato in: {`{{specializzazione}}`}</div>
                  <div className="diff-line"> </div>
                  <div className="diff-line">Quando rispondi:</div>
                  <div className="diff-line add">1. Usa type hints completi (PEP 484, PEP 695)</div>
                  <div className="diff-line add">2. Preferisci Pydantic v2 per validation</div>
                  <div className="diff-line">3. Scrivi docstring stile {`{{stile_docstring}}`}</div>
                  <div className="diff-line">4. Suggerisci sempre test pytest</div>
                  <div className="diff-line"> </div>
                  <div className="diff-line add">{`{{import "convenzioni/pep8-strict"}}`}</div>
                </div>
              </div>
            ) : (
              <div>
                {(prompt.body || '').split('\n').map((l, i) => <div key={i} className="diff-line">{l || ' '}</div>)}
              </div>
            )}
          </div>
        </div>
      </div>
    </Modal>
  );
}

// ---------- Insight ----------
function InsightModal({ open, onClose }) {
  return (
    <Modal open={open} onClose={onClose} title="Insight" sub="Statistiche del vault" width="820px">
      <div style={{ marginBottom: 18 }}>
        <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--text-strong)', marginBottom: 10 }}>Panoramica</div>
        <div className="stat-grid">
          {[{n:8,l:'Prompt attivi'},{n:6,l:'Tag'},{n:8,l:'Creati 30g'},{n:6,l:'Aggiornati 30g'},{n:14,l:'Versioni'},{n:0,l:'Cestinati'}].map((s,i) => (
            <div key={i} className="stat-card"><div className="num">{s.n}</div><div className="lbl">{s.l}</div></div>
          ))}
        </div>
      </div>
      <div style={{ marginBottom: 18 }}>
        <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--text-strong)', marginBottom: 10 }}>Top prompt usati (30g)</div>
        {[
          {n:'Email cold outreach B2B', c:154},
          {n:'Esperto devel Python', c:89},
          {n:'Code review checklist', c:67},
          {n:'Esperto devel Python (B) (fork)', c:47}
        ].map((p,i) => (
          <div key={i} className="bar-row">
            <span className="lbl">{p.n}</span>
            <div className="bar"><i style={{width: `${p.c/154*100}%`}}></i></div>
            <span className="num">{p.c}</span>
          </div>
        ))}
      </div>
      <div style={{ marginBottom: 18 }}>
        <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--text-strong)', marginBottom: 10 }}>Distribuzione per modello target</div>
        {[{n:'Claude Sonnet',c:3,p:50},{n:'Claude Opus',c:2,p:33},{n:'GPT-4',c:1,p:17},{n:'(non specificato)',c:2,p:33}].map((p,i) => (
          <div key={i} className="bar-row">
            <span className="lbl">{p.n}</span>
            <div className="bar"><i style={{width: `${p.p}%`}}></i></div>
            <span className="num">{p.c}</span>
          </div>
        ))}
      </div>
      <div>
        <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--text-strong)', marginBottom: 10 }}>Lint health</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: 12, background: 'var(--success-soft)', border: '1px solid var(--border-subtle)', borderRadius: 8 }}>
          <span style={{ fontSize: 28, fontWeight: 700, fontFamily: 'var(--font-mono)', color: 'var(--success)' }}>87%</span>
          <div>
            <div style={{ fontSize: 13, color: 'var(--text-strong)', fontWeight: 600 }}>prompt senza issue</div>
            <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 2 }}>Issue residue: 1 STY001 · 1 IMP004 (informativo)</div>
          </div>
        </div>
      </div>
    </Modal>
  );
}

// ---------- Regressioni ----------
function RegressioniModal({ open, onClose }) {
  return (
    <Modal open={open} onClose={onClose} title="Regressioni" sub="Drift di output golden tra modelli e versioni" width="min(1200px, 92vw)"
      footer={
        <React.Fragment>
          <span style={{ fontSize: 12, color: 'var(--text-muted)' }}>Aggiornato: ora · 3 prompt · 5 provider</span>
          <span style={{ flex: 1 }}></span>
          <select className="field-select" style={{ width: 'auto' }}><option>Ultimi 30 giorni</option><option>7 giorni</option><option>90 giorni</option><option>365 giorni</option></select>
          <button className="btn-secondary"><Icon name="export" size={12} /> Esporta CSV</button>
        </React.Fragment>
      }>
      <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
        <thead>
          <tr style={{ background: 'var(--bg-surface)', borderBottom: '1px solid var(--border-default)' }}>
            <th style={{ textAlign: 'left', padding: '8px 10px', fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Prompt × Provider</th>
            <th style={{ textAlign: 'left', padding: '8px 10px', fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Modello</th>
            <th style={{ padding: '8px 10px', fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Pass</th>
            <th style={{ padding: '8px 10px', fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Drift</th>
            <th style={{ padding: '8px 10px', fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Sparkline 30g</th>
            <th style={{ padding: '8px 10px', fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Ultima</th>
          </tr>
        </thead>
        <tbody>
          {[
            { p: 'Esperto devel Python (B) (fork)', m: 'claude-sonnet-4.7', pass: '2/3', drift: 'recente', driftKind: 'warn', last: '14m fa' },
            { p: 'Esperto devel Python (B) (fork)', m: 'gpt-4.1', pass: '3/3', drift: 'stabile', driftKind: 'ok', last: '1g fa' },
            { p: 'Code review checklist', m: 'claude-opus-4.7', pass: '5/5', drift: 'stabile', driftKind: 'ok', last: '3g fa' },
            { p: 'Code review checklist', m: 'gpt-4.1', pass: '4/5', drift: 'lieve', driftKind: 'warn', last: '3g fa' },
            { p: 'Email cold outreach B2B', m: 'gpt-4.1', pass: '1/3', drift: 'critico', driftKind: 'err', last: '6g fa' }
          ].map((r, i) => (
            <tr key={i} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
              <td style={{ padding: '8px 10px', color: 'var(--text-strong)', fontWeight: 500 }}>{r.p}</td>
              <td style={{ padding: '8px 10px', fontFamily: 'var(--font-mono)', color: 'var(--text-muted)' }}>{r.m}</td>
              <td style={{ padding: '8px 10px', textAlign: 'center', fontFamily: 'var(--font-mono)' }}>{r.pass}</td>
              <td style={{ padding: '8px 10px', textAlign: 'center' }}>
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 4, fontSize: 11, padding: '2px 6px', borderRadius: 4,
                  background: r.driftKind === 'err' ? 'var(--danger-soft)' : r.driftKind === 'warn' ? 'var(--warning-soft)' : 'var(--success-soft)',
                  color: r.driftKind === 'err' ? 'var(--danger)' : r.driftKind === 'warn' ? 'var(--warning)' : 'var(--success)' }}>
                  {r.drift}
                </span>
              </td>
              <td style={{ padding: '8px 10px' }}>
                <svg width="120" height="20" viewBox="0 0 120 20"><polyline points="0,12 15,8 30,10 45,6 60,9 75,7 90,11 105,5 120,8" fill="none" stroke="var(--accent-team)" strokeWidth="1.5"/></svg>
              </td>
              <td style={{ padding: '8px 10px', fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)' }}>{r.last}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </Modal>
  );
}

// ---------- Impostazioni ----------
function ImpostazioniModal({ open, onClose, density, onDensity, previewLines, onPreviewLines, theme, onTheme, tone, onTone }) {
  const [section, setSection] = useStateM('aspetto');
  const sections = [
    { id: 'account', l: 'Account', i: 'shield' },
    { id: 'aspetto', l: 'Aspetto', i: 'eye' },
    { id: 'lista', l: 'Vista lista prompt', i: 'sidebar' },
    { id: 'vault', l: 'Vault', i: 'lock' },
    { id: 'hotkey', l: 'Scorciatoie', i: 'kbd' },
    { id: 'ricerca', l: 'Ricerca semantica', i: 'cpu' },
    { id: 'provider', l: 'Provider AI', i: 'send' },
    { id: 'linter', l: 'Linter', i: 'alert-triangle' },
    { id: 'audit', l: 'Registro attività', i: 'activity' },
    { id: 'info', l: 'Informazioni', i: 'info' }
  ];

  return (
    <Modal open={open} onClose={onClose} title="Impostazioni" width="900px"
      footer={
        <React.Fragment>
          <span style={{ fontSize: 11, color: 'var(--text-subtle)', fontFamily: 'var(--font-mono)' }}>v0.8.0-beta · vault: ~/Library/PromptVault</span>
          <span style={{ flex: 1 }}></span>
          <button className="btn-ghost" onClick={onClose}>Chiudi</button>
        </React.Fragment>
      }>
      <div className="settings-grid">
        <div className="settings-nav">
          {sections.map(s => (
            <NavItem key={s.id} icon={s.i} name={s.l} active={section === s.id} onClick={() => setSection(s.id)} />
          ))}
        </div>
        <div className="settings-content">
          {section === 'lista' && (
            <React.Fragment>
              <h3 style={{ fontSize: 16, fontWeight: 600, color: 'var(--text-strong)', margin: '0 0 4px' }}>Vista lista prompt</h3>
              <p style={{ fontSize: 12, color: 'var(--text-muted)', margin: '0 0 16px' }}>Densità della seconda colonna (lista prompt). Cambia in tempo reale.</p>
              <div className="setting-row">
                <div>
                  <div className="lbl">Densità lista</div>
                  <div className="desc">Compatta = solo titolo + tag · Comoda = + descrizione · Anteprima = + N righe del body</div>
                </div>
                <div className="segmented">
                  <button className={density === 'dense' ? 'on' : ''} onClick={() => onDensity('dense')}>Compatta</button>
                  <button className={density === 'comfy' ? 'on' : ''} onClick={() => onDensity('comfy')}>Comoda</button>
                  <button className={density === 'preview' ? 'on' : ''} onClick={() => onDensity('preview')}>Anteprima</button>
                </div>
              </div>
              <div className="setting-row">
                <div>
                  <div className="lbl">Righe di anteprima del body</div>
                  <div className="desc">Quante righe del prompt mostrare in modalità Anteprima.</div>
                </div>
                <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
                  <input type="range" min="1" max="8" step="1" value={previewLines} onChange={e => onPreviewLines(+e.target.value)} />
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 13, color: 'var(--text-strong)', minWidth: 24 }}>{previewLines}</span>
                </div>
              </div>
              <div className="setting-row">
                <div><div className="lbl">Mostra conteggio uso nella lista</div><div className="desc">Numero accanto all'icona ▶.</div></div>
                <span className="toggle on"></span>
              </div>
              <div className="setting-row">
                <div><div className="lbl">Mostra ultima modifica</div><div className="desc">Timestamp relativo accanto al titolo.</div></div>
                <span className="toggle on"></span>
              </div>
            </React.Fragment>
          )}

          {section === 'aspetto' && (
            <React.Fragment>
              <h3 style={{ fontSize: 16, fontWeight: 600, color: 'var(--text-strong)', margin: '0 0 4px' }}>Aspetto</h3>
              <p style={{ fontSize: 12, color: 'var(--text-muted)', margin: '0 0 16px' }}>Tema e palette dei colori dell'app.</p>
              <div className="setting-row">
                <div><div className="lbl">Tema</div><div className="desc">Scuro / chiaro / segui il sistema.</div></div>
                <div className="segmented">
                  <button className={theme === 'dark' ? 'on' : ''} onClick={() => onTheme('dark')}>Scuro</button>
                  <button className={theme === 'light' ? 'on' : ''} onClick={() => onTheme('light')}>Chiaro</button>
                  <button onClick={() => onTheme('dark')}>Auto</button>
                </div>
              </div>
              <div className="setting-row">
                <div><div className="lbl">Tono palette</div><div className="desc">Temperatura della grigia neutra.</div></div>
                <div className="segmented">
                  <button className={tone === 'zinc' ? 'on' : ''} onClick={() => onTone('zinc')}>Zinc</button>
                  <button className={tone === 'slate' ? 'on' : ''} onClick={() => onTone('slate')}>Slate</button>
                  <button className={tone === 'stone' ? 'on' : ''} onClick={() => onTone('stone')}>Stone</button>
                </div>
              </div>
              <div className="setting-row">
                <div><div className="lbl">Densità globale</div><div className="desc">Padding generale dell'interfaccia.</div></div>
                <div className="segmented"><button className="on">Compatto</button><button>Standard</button></div>
              </div>
            </React.Fragment>
          )}

          {section === 'vault' && (
            <React.Fragment>
              <h3 style={{ fontSize: 16, fontWeight: 600, color: 'var(--text-strong)', margin: '0 0 4px' }}>Vault</h3>
              <p style={{ fontSize: 12, color: 'var(--text-muted)', margin: '0 0 16px' }}>Locale, cifrato AES-256 con SQLCipher · Argon2id.</p>
              <div className="setting-row">
                <div><div className="lbl">Posizione vault</div><div className="desc"><code style={{fontFamily:'var(--font-mono)', fontSize: 11}}>~/Library/PromptVault/pap-vault.db</code></div></div>
                <button className="btn-secondary"><Icon name="folder" size={12} /> Mostra in Finder</button>
              </div>
              <div className="setting-row">
                <div><div className="lbl">Cambia password</div><div className="desc">Richiede la password attuale.</div></div>
                <button className="btn-secondary">Cambia…</button>
              </div>
              <div className="setting-row">
                <div><div className="lbl">Auto-blocco dopo inattività</div><div className="desc">Blocca il vault dopo N minuti senza interazione.</div></div>
                <select className="field-select" style={{ width: 'auto' }}><option>5 min</option><option>15 min</option><option>30 min</option><option>Mai</option></select>
              </div>
              <div className="setting-row">
                <div><div className="lbl">Esporta vault completo</div><div className="desc">JSON o Markdown bundle, AES opzionale.</div></div>
                <button className="btn-secondary"><Icon name="export" size={12} /> Esporta…</button>
              </div>
              <div className="setting-row">
                <div><div className="lbl">Elimina vault</div><div className="desc">Operazione irreversibile.</div></div>
                <button className="btn-danger">Elimina vault…</button>
              </div>
            </React.Fragment>
          )}

          {!['lista','aspetto','vault'].includes(section) && (
            <div style={{ color: 'var(--text-muted)', fontSize: 13, padding: '20px 0' }}>
              <div style={{ fontSize: 14, color: 'var(--text-strong)', fontWeight: 600, marginBottom: 6 }}>{sections.find(s=>s.id===section)?.l}</div>
              Sezione disponibile in produzione · contenuto identico al brief Sezione 7.
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
}

// ---------- Command Palette ----------
function Palette({ open, onClose, onSelect }) {
  const [q, setQ] = useStateM('');
  const filtered = useMemoM(() => {
    if (!q.trim()) return window.PROMPTS.slice(0, 6);
    return window.PROMPTS.filter(p => p.title.toLowerCase().includes(q.toLowerCase()));
  }, [q]);

  useEffectM(() => { if (open) setQ(''); }, [open]);

  if (!open) return null;
  return (
    <div className="modal-scrim" onClick={onClose}>
      <div className="palette" onClick={e => e.stopPropagation()}>
        <div className="palette-input">
          <Icon name="search" size={15} />
          <input autoFocus placeholder="Cerca prompt, tag o azioni…" value={q} onChange={e => setQ(e.target.value)} onKeyDown={e => { if (e.key === 'Escape') onClose(); }} />
          <kbd>esc</kbd>
        </div>
        <div className="palette-results">
          <div className="palette-section">{q ? 'Risultati' : 'Recenti'}</div>
          {filtered.map((p, i) => (
            <div key={p.id} className={`palette-row ${i === 0 ? 'active' : ''}`} onClick={() => { onSelect(p.id); onClose(); }}>
              <span className="glyph"><Icon name={p.visibility === 'private' ? 'lock' : 'users'} size={12} /></span>
              <span className="label">{p.title}</span>
              <span className="sub">{p.folder} · {p.target}</span>
            </div>
          ))}
          <div className="palette-section">Azioni</div>
          <div className="palette-row"><span className="glyph"><Icon name="plus" size={12} /></span><span className="label">Nuovo prompt</span><span className="sub">⌘N</span></div>
        </div>
        <div className="palette-foot">
          <span className="seg"><kbd>↑</kbd><kbd>↓</kbd> naviga</span>
          <span className="seg"><kbd>↵</kbd> apri</span>
          <span className="seg"><kbd>⌃↵</kbd> compila & copia</span>
          <span style={{ marginLeft: 'auto' }} className="seg"><kbd>esc</kbd> chiudi</span>
        </div>
      </div>
    </div>
  );
}

window.Modal = Modal;
window.CompilaModal = CompilaModal;
window.CronologiaModal = CronologiaModal;
window.InsightModal = InsightModal;
window.RegressioniModal = RegressioniModal;
window.ImpostazioniModal = ImpostazioniModal;
window.Palette = Palette;
