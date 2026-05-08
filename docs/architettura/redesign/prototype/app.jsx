// =====================================================
// PromptVault — Redesign App root
// =====================================================
const { useState: useS, useEffect: useE, useMemo: useM, useRef: useR } = React;

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "dark",
  "tone": "zinc",
  "density": "comfy",
  "previewLines": 3,
  "sidebarCollapsed": false,
  "rightRailCollapsed": false,
  "view": "tutti"
}/*EDITMODE-END*/;

function App() {
  // tweakable state — uses tweaks-panel.jsx hook
  const [tweaks, setTweak] = useTweaks(TWEAK_DEFAULTS);
  const [theme, setTheme] = useS(tweaks.theme);
  const [tone, setTone] = useS(tweaks.tone);
  const [density, setDensity] = useS(tweaks.density);
  const [previewLines, setPreviewLines] = useS(tweaks.previewLines);
  const [sidebarCollapsed, setSidebarCollapsed] = useS(tweaks.sidebarCollapsed);
  const [rightRailCollapsed, setRightRailCollapsed] = useS(tweaks.rightRailCollapsed);
  const [view, setView] = useS(tweaks.view);

  useE(() => { document.documentElement.dataset.theme = theme; setTweak('theme', theme); }, [theme]);
  useE(() => { document.documentElement.dataset.tone = tone; setTweak('tone', tone); }, [tone]);
  useE(() => { setTweak('density', density); }, [density]);
  useE(() => { setTweak('previewLines', previewLines); }, [previewLines]);
  useE(() => { setTweak('sidebarCollapsed', sidebarCollapsed); }, [sidebarCollapsed]);
  useE(() => { setTweak('rightRailCollapsed', rightRailCollapsed); }, [rightRailCollapsed]);
  useE(() => { setTweak('view', view); }, [view]);

  // Selection / detail state
  const [activeId, setActiveId] = useS('p1');
  const [tab, setTab] = useS('editor');
  const [modal, setModal] = useS(null); // 'compila' 'cronologia' 'insight' 'regressioni' 'impostazioni'
  const [paletteOpen, setPaletteOpen] = useS(false);
  const [activeFolder, setActiveFolder] = useS(null);
  const [activeTag, setActiveTag] = useS(null);
  const [modelTarget, setModelTarget] = useS('Tutti');
  const [sort, setSort] = useS('Recenti');
  const [dirty, setDirty] = useS(false);
  const [saved, setSaved] = useS('14s fa');

  // Resizable columns
  const [colSidebar, setColSidebar] = useS(248);
  const [colList, setColList] = useS(360);
  const [rightRail, setRightRail] = useS(300);

  const dragRef = useR(null);
  useE(() => {
    const onMove = (e) => {
      if (!dragRef.current) return;
      const { which, startX, startVal, max } = dragRef.current;
      const dx = e.clientX - startX;
      const next = Math.max(180, Math.min(max, startVal + dx));
      if (which === 'sidebar') setColSidebar(next);
      if (which === 'list') setColList(next);
      if (which === 'rail') setRightRail(Math.max(220, Math.min(480, startVal - dx)));
    };
    const onUp = () => { dragRef.current = null; document.body.style.cursor=''; document.querySelectorAll('.resizer.dragging').forEach(r => r.classList.remove('dragging')); };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    return () => { window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
  }, []);
  const startDrag = (which, val, max, ev) => {
    dragRef.current = { which, startX: ev.clientX, startVal: val, max };
    document.body.style.cursor = 'col-resize';
    ev.currentTarget.classList.add('dragging');
  };

  // Hotkeys
  useE(() => {
    const onKey = (e) => {
      const cmdShift = (e.metaKey || e.ctrlKey) && e.shiftKey;
      const cmd = (e.metaKey || e.ctrlKey) && !e.shiftKey;
      if (cmdShift && e.key.toLowerCase() === 'p') { e.preventDefault(); setPaletteOpen(true); }
      if (cmd && e.key.toLowerCase() === 'k') { e.preventDefault(); setPaletteOpen(true); }
      if (cmd && e.key === ',') { e.preventDefault(); setModal('impostazioni'); }
      if (cmd && e.key === 'Enter') { e.preventDefault(); setModal('compila'); }
      if (cmd && e.key.toLowerCase() === 's') { e.preventDefault(); setDirty(false); setSaved('ora'); }
      if (e.key === 'Escape') { setPaletteOpen(false); setModal(null); }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // Filtered prompts based on view
  const filtered = useM(() => {
    let list = window.PROMPTS;
    if (view === 'recenti') list = list.slice(0, 6);
    if (view === 'preferiti') list = list.filter(p => p.favorite);
    if (view === 'privati') list = list.filter(p => p.visibility === 'private');
    if (view === 'team') list = list.filter(p => p.visibility === 'team');
    if (activeFolder) list = list.filter(p => p.folder?.startsWith(activeFolder));
    if (activeTag) list = list.filter(p => p.tags.includes(activeTag));
    if (modelTarget && modelTarget !== 'Tutti') list = list.filter(p => p.target === modelTarget);
    return list;
  }, [view, activeFolder, activeTag, modelTarget]);

  const active = window.PROMPTS.find(p => p.id === activeId);

  const filterChips = [];
  if (activeTag) filterChips.push({ id: 'tag', label: '#' + activeTag, kind: 'applied' });
  if (activeFolder) filterChips.push({ id: 'folder', label: activeFolder, kind: 'applied' });
  if (modelTarget !== 'Tutti') filterChips.push({ id: 'target', label: modelTarget, kind: 'applied' });

  const clearFilter = (id) => {
    if (id === 'tag') setActiveTag(null);
    if (id === 'folder') setActiveFolder(null);
    if (id === 'target') setModelTarget('Tutti');
  };

  const viewTitle = view === 'recenti' ? 'Recenti' : view === 'preferiti' ? 'Preferiti' : view === 'privati' ? 'Privati' : view === 'team' ? 'Team' : view === 'tutti' ? 'Tutti i prompt' : 'Prompt';

  const onUpdatePrompt = (patch) => { setDirty(true); /* mock */ };

  const cycleDensity = () => {
    const order = ['dense', 'comfy', 'preview'];
    const idx = order.indexOf(density);
    setDensity(order[(idx + 1) % order.length]);
  };

  const newPrompt = () => {
    setActiveId('new');
    setTab('editor');
  };

  return (
    <div className="app">
      <TitleBar
        onPalette={() => setPaletteOpen(true)}
        theme={theme} onTheme={setTheme}
        onSettings={() => setModal('impostazioni')}
      />

      <div className="body" data-sidebar-collapsed={sidebarCollapsed} data-list-collapsed={colList === 0} style={{ ['--col-sidebar']: colSidebar + 'px', ['--col-list']: colList + 'px' }}>
        <Sidebar
          collapsed={sidebarCollapsed}
          onToggle={() => setSidebarCollapsed(c => !c)}
          view={view} onView={setView}
          modelTarget={modelTarget} onModelTarget={setModelTarget}
          activeFolder={activeFolder} onFolder={setActiveFolder}
          activeTag={activeTag} onTag={setActiveTag}
          onOpenModal={setModal}
        />
        <div className="resizer" onMouseDown={(e) => startDrag('sidebar', colSidebar, 360, e)} />

        <ListPane
          prompts={filtered}
          view={view} viewTitle={viewTitle}
          density={density} previewLines={previewLines}
          sort={sort} onSort={() => { const order = ['Recenti','Popolari','Migliori','A-Z']; setSort(order[(order.indexOf(sort)+1)%order.length]); }}
          activeId={activeId} onSelect={setActiveId}
          onNew={newPrompt}
          onToggleFav={() => {}}
          filterChips={filterChips}
          onClearFilter={clearFilter}
          onToggleDensity={cycleDensity}
          onCollapse={(action) => setColList(action === 'restore' ? 360 : 0)}
        />
        <div className="resizer" onMouseDown={(e) => startDrag('list', colList, 480, e)} />

        <DetailPane
          prompt={active}
          tab={tab} onTab={setTab}
          dirty={dirty} saved={saved}
          rightRailCollapsed={rightRailCollapsed}
          onToggleRail={() => setRightRailCollapsed(c => !c)}
          rightRail={rightRail}
          onResizeRail={(val, max, ev) => startDrag('rail', rightRail, 480, ev)}
          onCompila={() => setModal('compila')}
          onCronologia={() => { setTab('cronologia'); }}
          onUpdate={onUpdatePrompt}
          setDirty={setDirty}
        />
      </div>

      <StatusBar prompt={active} dirty={dirty} saved={saved} onPalette={() => setPaletteOpen(true)} />

      {/* Modals */}
      <CompilaModal open={modal === 'compila'} onClose={() => setModal(null)} prompt={active} />
      <InsightModal open={modal === 'insight'} onClose={() => setModal(null)} />
      <RegressioniModal open={modal === 'regressioni'} onClose={() => setModal(null)} />
      <ImpostazioniModal open={modal === 'impostazioni'} onClose={() => setModal(null)}
        density={density} onDensity={setDensity}
        previewLines={previewLines} onPreviewLines={setPreviewLines}
        theme={theme} onTheme={setTheme}
        tone={tone} onTone={setTone} />
      <Palette open={paletteOpen} onClose={() => setPaletteOpen(false)} onSelect={setActiveId} />

      {/* Tweaks panel */}
      <TweaksPanel title="Tweaks">
        <TweakSection title="Layout">
          <TweakToggle label="Sidebar compressa" value={sidebarCollapsed} onChange={setSidebarCollapsed} />
          <TweakToggle label="Pannello dx compresso" value={rightRailCollapsed} onChange={setRightRailCollapsed} />
        </TweakSection>
        <TweakSection title="Vista lista prompt">
          <TweakRadio label="Densità" value={density} onChange={setDensity} options={[
            { value: 'dense', label: 'Compatta' },
            { value: 'comfy', label: 'Comoda' },
            { value: 'preview', label: 'Anteprima' }
          ]} />
          <TweakSlider label="Righe anteprima" min={1} max={8} step={1} value={previewLines} onChange={setPreviewLines} />
        </TweakSection>
        <TweakSection title="Aspetto">
          <TweakRadio label="Tema" value={theme} onChange={setTheme} options={[
            { value: 'dark', label: 'Scuro' },
            { value: 'light', label: 'Chiaro' }
          ]} />
          <TweakSelect label="Tono" value={tone} onChange={setTone} options={[
            { value: 'zinc', label: 'Zinc · neutro' },
            { value: 'slate', label: 'Slate · freddo' },
            { value: 'stone', label: 'Stone · caldo' }
          ]} />
        </TweakSection>
        <TweakSection title="Demo">
          <TweakButton onClick={() => setModal('compila')}>Apri Compila</TweakButton>
          <TweakButton onClick={() => setModal('insight')}>Apri Insight</TweakButton>
          <TweakButton onClick={() => setModal('regressioni')}>Apri Regressioni</TweakButton>
          <TweakButton onClick={() => setPaletteOpen(true)}>Apri Command Palette</TweakButton>
        </TweakSection>
      </TweaksPanel>
    </div>
  );
}

// ---------- Detail pane (lives in app.jsx because it stitches subcomponents) ----------
function DetailPane({ prompt, tab, onTab, dirty, saved, rightRailCollapsed, onToggleRail, rightRail, onResizeRail, onCompila, onCronologia, onUpdate, setDirty }) {
  if (!prompt) {
    return (
      <section className="detail">
        <div className="detail-empty">
          <Icon name="file" size={64} />
          <div className="t">Seleziona un prompt</div>
          <div className="s">Clicca un prompt nella lista per vedere ed editare il body, oppure crea un nuovo prompt con ⌘N.</div>
        </div>
      </section>
    );
  }

  const lintCount = (prompt.lint || []).filter(l => l.level !== 'info').length;
  const goldenCount = (prompt.goldens || []).length;
  const varsCount = (prompt.placeholders || []).length;
  const historyCount = (prompt.history || []).length;

  return (
    <section className="detail">
      <div className="detail-header">
        <div className="detail-titlerow">
          <div className="detail-title-block">
            <input className="detail-title" defaultValue={prompt.title} onChange={() => setDirty(true)} />
            <textarea className="detail-desc" rows={1} defaultValue={prompt.desc} onChange={() => setDirty(true)} placeholder="Aggiungi una descrizione (opzionale)…" />
          </div>
          <div className="detail-actions">
            <button className="icon-btn" title="Preferito"><Icon name={prompt.favorite ? 'star-fill' : 'star'} size={15} /></button>
            <span style={{ width: 1, height: 18, background: 'var(--border-subtle)', margin: '0 4px' }}></span>
            <button className="btn-ghost" title="Crea fork in workspace privato"><Icon name="fork" size={12} /> Fork</button>
            <button className="btn-ghost" title="Esporta come Markdown con front-matter"><Icon name="export" size={12} /> Esporta MD</button>
            <button className="btn-primary" onClick={onCompila} title="Compila variabili e copia in clipboard"><Icon name="play" size={11} /> Compila <kbd style={{marginLeft:4, color:'rgba(0,0,0,0.4)'}}>⌃↵</kbd></button>
            <button className="btn-ghost icon-toggle meta-toggle" title={rightRailCollapsed ? 'Mostra metadati' : 'Nascondi metadati'} onClick={onToggleRail}>
              <Icon name={rightRailCollapsed ? 'panel-rt' : 'sidebar-right'} size={14} />
              <span style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)' }}>Meta</span>
            </button>
          </div>
        </div>

        <div className="detail-meta">
          <span className={`meta-chip ${prompt.visibility}`}>
            <Icon name={prompt.visibility === 'private' ? 'lock' : 'users'} size={11} className="glyph" />
            {prompt.visibility === 'private' ? 'Privato' : 'Team'}
          </span>
          <span className="meta-chip"><span className="lbl">cartella</span> {prompt.folder || '/'}</span>
          <span className="meta-chip"><span className="lbl">target</span> {prompt.target}</span>
          <span className="meta-chip"><span className="lbl">var.</span> {prompt.isVariant || 'A'} {prompt.variants ? `· ${prompt.variants.length} totali` : ''}</span>
          {prompt.forkOf && <span className="meta-chip" style={{ color: 'var(--info)' }}><Icon name="fork" size={11} className="glyph"/>fork di "{prompt.forkOf}"</span>}
          <span style={{ flex: 1 }}></span>
          <span className="meta-chip"><span className="lbl">usato</span> {prompt.useCount || 0}×</span>
          <span className="meta-chip"><span className="lbl">aggiornato</span> {prompt.updated}</span>
        </div>

        <DetailTabs tab={tab} onTab={onTab}
          lintCount={lintCount} goldenCount={goldenCount}
          varsCount={varsCount} historyCount={historyCount} />
      </div>

      <div className="detail-body" data-rail-collapsed={rightRailCollapsed} style={{ ['--right-rail']: rightRail + 'px' }}>
        <div className="editor-col">
          {tab === 'editor' && (
            <React.Fragment>
              <div className="editor-toolbar">
                <button className="icon-btn" title="Grassetto · ⌘B"><Icon name="md-bold" size={13} /></button>
                <button className="icon-btn" title="Corsivo · ⌘I"><Icon name="md-italic" size={13} /></button>
                <span className="sep"></span>
                <button className="icon-btn" title="Heading 1 · ⌘1"><Icon name="md-h1" size={13} /></button>
                <button className="icon-btn" title="Heading 2 · ⌘2"><Icon name="md-h2" size={13} /></button>
                <span className="sep"></span>
                <button className="icon-btn" title="Lista puntata"><Icon name="md-ul" size={13} /></button>
                <button className="icon-btn" title="Lista numerata"><Icon name="md-ol" size={13} /></button>
                <button className="icon-btn" title="Citazione"><Icon name="md-quote" size={13} /></button>
                <span className="sep"></span>
                <button className="icon-btn" title="Codice inline · ⌘E"><Icon name="md-code" size={13} /></button>
                <button className="icon-btn" title="Blocco codice"><Icon name="md-codeblock" size={13} /></button>
                <button className="icon-btn" title="Link · ⌘K"><Icon name="md-link" size={13} /></button>
                <button className="icon-btn" title="Separatore"><Icon name="md-hr" size={13} /></button>
                <span className="sep"></span>
                <button className="btn-ghost" title="Inserisci segnaposto"><Icon name="plus" size={11} /> {`{{var}}`}</button>
                <button className="btn-ghost" title="Inserisci import"><Icon name="fork" size={11} /> import</button>
                <span className="sep"></span>
                <button className="icon-btn" title="Cerca nel body · ⌘F"><Icon name="search" size={12} /></button>
                <span className="indicator">
                  {dirty ? <span className="dot-dirty"></span> : <span className="dot-saved"></span>}
                  {dirty ? 'Non salvato' : `Salvato ${saved}`}
                  <span style={{ width: 1, height: 12, background: 'var(--border-subtle)' }}></span>
                  <span>L 1, C 1 · {(prompt.body||'').length} char · ~{Math.round((prompt.body||'').length/4)} tok</span>
                </span>
              </div>
              <MockEditor body={prompt.body || ''} dirty={dirty} onChange={() => setDirty(true)} />
            </React.Fragment>
          )}

          {tab === 'anteprima' && (
            <div className="editor-scroll" style={{ padding: '24px 40px' }}>
              <div style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: 8 }}>
                Anteprima · valori di default applicati
              </div>
              <div style={{ background: 'var(--bg-surface)', border: '1px solid var(--border-subtle)', borderRadius: 8, padding: 18, fontFamily: 'var(--font-mono)', fontSize: 13, lineHeight: 1.65, whiteSpace: 'pre-wrap' }}>
                {(prompt.body || '').replace(/\{\{(\w+)\}\}/g, (_, k) => {
                  const ph = (prompt.placeholders||[]).find(p => p.name === k);
                  return ph?.default ? ph.default : `[${k}]`;
                })}
              </div>
            </div>
          )}

          {tab === 'diagnosi' && (
            <div className="editor-scroll" style={{ padding: '16px 24px' }}>
              <div style={{ maxWidth: 720 }}>
                <div style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: 12 }}>
                  Linter · {(prompt.lint||[]).length} segnalazioni
                </div>
                {(prompt.lint || []).map((l, i) => (
                  <div className="lint-row" key={i}>
                    <span className={`level ${l.level === 'warn' ? 'warn' : l.level === 'err' ? 'err' : 'info'}`}></span>
                    <div>
                      <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 2 }}>
                        <span className="code">{l.code}</span>
                        {l.line && <span className="line">L {l.line}</span>}
                      </div>
                      <div className="msg">{l.msg}</div>
                    </div>
                    <button className="btn-ghost" style={{ height: 22, fontSize: 11 }}>Vai</button>
                  </div>
                ))}
                {(prompt.lint || []).length === 0 && <div style={{ fontSize: 13, color: 'var(--text-muted)' }}>Nessuna segnalazione.</div>}
              </div>
            </div>
          )}

          {tab === 'test' && (
            <div className="editor-scroll" style={{ padding: '16px 24px' }}>
              <div style={{ maxWidth: 760 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 12 }}>
                  <span style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Golden examples</span>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--text-muted)' }}>2/3 pass · ultimo run 14m fa</span>
                  <span style={{ flex: 1 }}></span>
                  <select className="field-select" style={{ width: 'auto', height: 26 }}><option>claude-sonnet-4.7</option><option>gpt-4.1</option><option>gemini-pro</option><option>ollama: llama3.2</option></select>
                  <button className="btn-secondary" style={{ height: 26 }}><Icon name="play-circle" size={12} /> Esegui tutti</button>
                  <button className="btn-ghost" style={{ height: 26 }}><Icon name="plus" size={11} /> Golden</button>
                </div>
                {(prompt.goldens || []).map(g => (
                  <div className="golden-row" key={g.id}>
                    <span className={`status ${g.status}`}>
                      <Icon name={g.status === 'pass' ? 'check' : g.status === 'fail' ? 'x' : 'play'} size={10} />
                    </span>
                    <div>
                      <div className="label">{g.label}</div>
                      <div className="meta">{g.model} · {g.last} · similarity: cosine ≥ 0.85</div>
                    </div>
                    <button className="btn-ghost" style={{ height: 24, fontSize: 11 }}>Dettagli</button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {tab === 'cronologia' && (
            <div className="editor-scroll" style={{ padding: 0 }}>
              <div style={{ display: 'grid', gridTemplateColumns: '260px 1fr', height: '100%', minHeight: 0 }}>
                <div style={{ borderRight: '1px solid var(--border-subtle)', overflowY: 'auto', padding: 8, background: 'var(--bg-surface)' }}>
                  {(prompt.history || []).map(h => (
                    <div key={h.v} className={`history-row ${h.head ? 'active' : ''}`}>
                      <div className="v">{h.v}{h.head && <span className="head-pill">TESTA</span>}</div>
                      <div className="author">
                        <span className={`av ${h.team ? 'team' : ''}`}>{h.authorInit}</span>
                        <span>{h.author}</span>
                        <span style={{ flex: 1 }}></span>
                        <span className="ts">{h.ts.split(' ')[1]}</span>
                      </div>
                      <div className="summary">{h.summary}</div>
                      <div className="delta"><span className="add">+{h.delta.add}</span><span className="del">−{h.delta.del}</span></div>
                    </div>
                  ))}
                </div>
                <div style={{ padding: 16, overflow: 'auto' }}>
                  <div style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Versione 3 (TESTA)</div>
                  <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--text-strong)', marginTop: 4 }}>Modifiche di Roberto M. · ieri 17:42</div>
                  <div style={{ marginTop: 12, fontFamily: 'var(--font-mono)', fontSize: 12.5, lineHeight: 1.7, whiteSpace: 'pre-wrap' }}>
                    <div className="diff-line">Sei un esperto sviluppatore Python con 30 anni di esperienza.</div>
                    <div className="diff-line"></div>
                    <div className="diff-line">Specializzato in: {`{{specializzazione}}`}</div>
                    <div className="diff-line"></div>
                    <div className="diff-line">Quando rispondi:</div>
                    <div className="diff-line del">- 1. Usa type hints completi (PEP 484)</div>
                    <div className="diff-line add">+ 1. Usa type hints completi (PEP 484, PEP 695)</div>
                    <div className="diff-line del">- 2. Preferisci dataclasses per validation</div>
                    <div className="diff-line add">+ 2. Preferisci Pydantic v2 per validation</div>
                    <div className="diff-line">3. Scrivi docstring stile {`{{stile_docstring}}`}</div>
                    <div className="diff-line">4. Suggerisci sempre test pytest</div>
                    <div className="diff-line"></div>
                    <div className="diff-line add">+ {`{{import "convenzioni/pep8-strict"}}`}</div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {tab === 'import' && (
            <div className="editor-scroll" style={{ padding: '16px 24px' }}>
              <div style={{ maxWidth: 760 }}>
                <div style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: 8 }}>Import composti</div>
                <div style={{ background: 'var(--bg-surface)', border: '1px solid var(--border-subtle)', borderRadius: 8, padding: 12, marginBottom: 16 }}>
                  {(prompt.imports || []).map(i => (
                    <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 0', borderTop: '1px solid var(--border-subtle)' }}>
                      <Icon name="fork" size={12} />
                      <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--info)', fontSize: 12 }}>{i}</span>
                      <span style={{ flex: 1 }}></span>
                      <button className="btn-ghost" style={{ height: 22, fontSize: 11 }}>Apri</button>
                      <button className="btn-ghost" style={{ height: 22, fontSize: 11 }}>Espandi</button>
                    </div>
                  ))}
                </div>
                <div style={{ fontSize: 11, fontFamily: 'var(--font-mono)', color: 'var(--text-subtle)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: 8 }}>Varianti A/B/Z</div>
                <div className="variants-row" style={{ marginBottom: 12 }}>
                  {(prompt.variants || []).map(v => (
                    <span key={v} className={`variant-pill ${v === prompt.isVariant ? 'active' : ''}`}>
                      <span className="label">Variante {v}</span>
                      <span className="sub">{v === prompt.isVariant ? '· corrente' : 'apri'}</span>
                    </span>
                  ))}
                  <button className="btn-secondary" style={{ height: 24 }}><Icon name="plus" size={11} /> Nuova variante</button>
                  <button className="btn-secondary" style={{ height: 24 }}><Icon name="split" size={11} /> Confronta tutte</button>
                </div>
              </div>
            </div>
          )}
        </div>

        {!rightRailCollapsed && tab === 'editor' && (
          <RightRail prompt={prompt} onUpdate={onUpdate} />
        )}
      </div>
    </section>
  );
}

// ---------- Status bar ----------
function StatusBar({ prompt, dirty, saved, onPalette }) {
  return (
    <div className="statusbar">
      <span className="seg" title="Vault locale cifrato AES-256 · SQLCipher · embeddings MiniLM-L12 in memoria">
        <span className="status-dot ok"></span>
        <Icon name="lock" size={11} />
        <span>vault locale</span>
      </span>
      {prompt && (
        <React.Fragment>
          <span className="sep"></span>
          <span className="seg" style={{ color: 'var(--text-strong)' }}>
            <Icon name={prompt.visibility === 'private' ? 'lock' : 'users'} size={11} />
            {prompt.title}
          </span>
        </React.Fragment>
      )}
      <div className="right">
        <span className="seg" style={{ color: dirty ? 'var(--warning)' : 'var(--text-subtle)' }}>
          {dirty ? <span className="dot-dirty"></span> : <span className="dot-saved"></span>}
          {dirty ? 'non salvato' : `salvato ${saved}`}
        </span>
        <span className="clickable seg" onClick={onPalette} title="Apri command palette"><kbd>⌃⇧P</kbd></span>
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
