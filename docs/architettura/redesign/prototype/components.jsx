// =====================================================
// Sub-components for PromptVault redesign
// =====================================================
const { useState, useEffect, useRef, useMemo } = React;

// ---------- Title bar ----------
function TitleBar({ onPalette, onTweaks, theme, onTheme, onSettings }) {
  return (
    <div className="titlebar">
      <div className="group left">
        <div className="brand"><span className="glyph">P</span> Prompt a Porter</div>
      </div>
      <div className="group center">
        <button className="tb-search" onClick={onPalette} title="⌃⇧P · Cerca prompt, tag o azioni">
          <Icon name="search" size={12} />
          <span className="placeholder">Cerca prompt, tag o azioni…</span>
          <kbd>⌃⇧P</kbd>
        </button>
      </div>
      <div className="group right">
        <button className="icon-btn" onClick={() => onTheme(theme === 'dark' ? 'light' : 'dark')} title="Tema">
          <Icon name={theme === 'dark' ? 'eye' : 'eye'} size={14} />
        </button>
        <button className="icon-btn" onClick={onSettings} title="Impostazioni ⌘,">
          <Icon name="settings" size={14} />
        </button>
        <div className="win-controls">
          <button className="win-btn" title="Minimizza"><Icon name="minimize" size={12} /></button>
          <button className="win-btn" title="Massimizza"><Icon name="square" size={11} /></button>
          <button className="win-btn close" title="Chiudi"><Icon name="x" size={12} /></button>
        </div>
      </div>
    </div>
  );
}

// ---------- Sidebar ----------
function NavGroup({ title, defaultCollapsed = false, addAction, children }) {
  const [collapsed, setCollapsed] = useState(defaultCollapsed);
  return (
    <div className="nav-group" data-collapsed={collapsed}>
      <button className="nav-group-header" onClick={() => setCollapsed(c => !c)}>
        <Icon name="chevron-down" size={10} className="chev" />
        <span className="label">{title}</span>
        {addAction && <span className="add" onClick={(e) => { e.stopPropagation(); addAction(); }}><Icon name="plus" size={11} /></span>}
      </button>
      <div className="nav-group-body">{children}</div>
    </div>
  );
}

function NavItem({ icon, name, count, dot, active, onClick, indent = 0, color }) {
  return (
    <button className={`nav-item ${active ? 'active' : ''} ${indent ? 'indent' + (indent > 1 ? '2' : '') : ''}`} onClick={onClick}>
      {dot ? <span className="dot" style={{ background: color || 'var(--text-muted)' }} /> : icon ? <span className="glyph"><Icon name={icon} size={14} /></span> : null}
      <span className="name">{name}</span>
      {count != null && <span className="count">{count}</span>}
    </button>
  );
}

function Sidebar({ collapsed, onToggle, view, onView, modelTarget, onModelTarget, onOpenModal, activeFolder, onFolder, activeTag, onTag }) {
  if (collapsed) {
    return (
      <aside className="sidebar">
        <div className="sidebar-mini">
          <button className="mini-btn" title="Apri sidebar" onClick={onToggle}><Icon name="chevrons-right" size={14} /></button>
          <div style={{ height: 8 }}></div>
          <button className={`mini-btn ${view==='recenti'?'active':''}`} title="Recenti" onClick={() => onView('recenti')}><Icon name="clock" size={14} /></button>
          <button className={`mini-btn ${view==='preferiti'?'active':''}`} title="Preferiti" onClick={() => onView('preferiti')}><Icon name="star" size={14} /></button>
          <button className={`mini-btn ${view==='tutti'?'active':''}`} title="Tutti i prompt" onClick={() => onView('tutti')}><Icon name="file" size={14} /></button>
          <button className={`mini-btn ${view==='privati'?'active':''}`} title="Privati" onClick={() => onView('privati')}><Icon name="lock" size={14} /></button>
          <button className={`mini-btn ${view==='team'?'active':''}`} title="Team" onClick={() => onView('team')}><Icon name="users" size={14} /></button>
          <div style={{ flex: 1 }}></div>
          <button className="mini-btn" title="Insight" onClick={() => onOpenModal('insight')}><Icon name="bar-chart" size={14} /></button>
          <button className="mini-btn" title="Regressioni" onClick={() => onOpenModal('regressioni')}><Icon name="activity" size={14} /></button>
          <button className="mini-btn" title="Impostazioni" onClick={() => onOpenModal('impostazioni')}><Icon name="settings" size={14} /></button>
        </div>
      </aside>
    );
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <div className="workspace">
          <div className="avatar">P</div>
          <div className="name">Personale</div>
          <span className="chev"><Icon name="chevron-down" size={11} /></span>
        </div>
        <button className="icon-btn sidebar-collapse" title="Comprimi sidebar" onClick={onToggle}>
          <Icon name="chevrons-left" size={14} />
        </button>
      </div>

      <div className="sidebar-scroll">
        <NavGroup title="Viste">
          <NavItem icon="clock" name="Recenti" count={6} active={view==='recenti'} onClick={() => onView('recenti')} />
          <NavItem icon="star" name="Preferiti" count={1} active={view==='preferiti'} onClick={() => onView('preferiti')} />
          <NavItem icon="file" name="Tutti i prompt" count={8} active={view==='tutti'} onClick={() => onView('tutti')} />
        </NavGroup>

        <NavGroup title="Visibilità">
          <NavItem dot color="var(--accent-private)" name="Privati" count={6} active={view==='privati'} onClick={() => onView('privati')} />
          <NavItem dot color="var(--accent-team)" name="Team" count={2} active={view==='team'} onClick={() => onView('team')} />
        </NavGroup>

        <NavGroup title="Cartelle" addAction={() => {}}>
          <NavItem icon="folder" name="Senza cartella" count={3} active={activeFolder==='/'} onClick={() => onFolder('/')} />
          <NavItem icon="folder-open" name="Ruoli" count={3} active={activeFolder==='Ruoli'} onClick={() => onFolder('Ruoli')} />
          <NavItem icon="folder" name="Developer" count={2} indent={1} onClick={() => onFolder('Ruoli/Developer')} />
          <NavItem icon="folder" name="Content" count={1} indent={1} onClick={() => onFolder('Ruoli/Content')} />
          <NavItem icon="folder" name="Email" count={1} active={activeFolder==='Email'} onClick={() => onFolder('Email')} />
          <NavItem icon="folder" name="Sandbox" count={2} active={activeFolder==='Sandbox'} onClick={() => onFolder('Sandbox')} />
        </NavGroup>

        <NavGroup title="Tag" addAction={() => {}}>
          {window.TAGS.map(t => (
            <NavItem key={t.id} dot color={t.color} name={t.name} count={t.count} active={activeTag===t.id} onClick={() => onTag(t.id)} />
          ))}
        </NavGroup>

        <NavGroup title="Modello target" defaultCollapsed>
          {window.TARGETS.map(t => (
            <NavItem key={t} name={t} active={modelTarget===t} onClick={() => onModelTarget(t)} />
          ))}
        </NavGroup>
      </div>

      <div className="sidebar-footer">
        <button className="nav-item" onClick={() => onOpenModal('insight')}><span className="glyph"><Icon name="bar-chart" size={13} /></span> <span className="name">Insight</span></button>
        <button className="nav-item" onClick={() => onOpenModal('regressioni')}><span className="glyph"><Icon name="activity" size={13} /></span> <span className="name">Regressioni</span></button>
      </div>
    </aside>
  );
}

// ---------- Lista prompt ----------
function PromptCard({ prompt, active, density, previewLines, onClick, onToggleFav }) {
  return (
    <div className={`prompt-card ${active ? 'active' : ''}`} onClick={onClick}>
      <span className={`vis ${prompt.visibility}`} title={prompt.visibility === 'private' ? 'Privato' : 'Team'} />
      <div className="body">
        <div className="row">
          <span className="title">{prompt.title}</span>
          <span className="meta-time">{prompt.updated.split('·')[0].trim()}</span>
        </div>
        {prompt.desc && <div className="desc">{prompt.desc}</div>}
        {density === 'preview' && (
          <div className="preview" style={{ ['--preview-lines']: previewLines }}>{prompt.body}</div>
        )}
        {prompt.tags && prompt.tags.length > 0 && (
          <div className="tags">
            {prompt.tags.map(t => {
              const tag = window.TAGS.find(x => x.id === t);
              return <span key={t} className="tag-pill"><span className="dot" style={{ background: tag?.color || 'var(--text-muted)' }} />{t}</span>;
            })}
          </div>
        )}
      </div>
      <div className="badges">
        <button className={`fav-star ${prompt.favorite ? 'on' : ''}`} onClick={(e) => { e.stopPropagation(); onToggleFav?.(prompt.id); }} style={{ background: 'transparent', border: 'none', padding: 0 }}>
          <Icon name={prompt.favorite ? 'star-fill' : 'star'} size={13} />
        </button>
        {prompt.useCount != null && (
          <span className="use-count" title={`Usato ${prompt.useCount} volte`}>
            <Icon name="play" size={9} /> {prompt.useCount}
          </span>
        )}
      </div>
    </div>
  );
}

function ListPane({ prompts, view, viewTitle, density, previewLines, sort, onSort, activeId, onSelect, onNew, onToggleFav, filterChips, onClearFilter, onToggleDensity, onCollapse }) {
  const [q, setQ] = useState('');
  const filtered = useMemo(() => {
    if (!q.trim()) return prompts;
    const qq = q.toLowerCase();
    return prompts.filter(p => p.title.toLowerCase().includes(qq) || (p.desc||'').toLowerCase().includes(qq) || (p.body||'').toLowerCase().includes(qq) || p.tags.some(t => t.toLowerCase().includes(qq)));
  }, [q, prompts]);

  return (
    <section className="list-pane" data-density={density}>
      <button className="list-restore" onClick={() => onCollapse?.('restore')} title="Mostra lista prompt">
        <Icon name="list" size={13} />
      </button>
      <div className="list-header">
        <div className="list-titlerow">
          <span className="list-title">{viewTitle}</span>
          <span className="list-count">{filtered.length}</span>
          <span className="list-sub">aggiornata ora</span>
          <button className="icon-btn list-collapse" title="Comprimi lista" onClick={() => onCollapse?.()}>
            <Icon name="chevrons-left" size={14} />
          </button>
        </div>

        <div className="list-toolbar">
          <div className="list-search">
            <Icon name="search" size={12} />
            <input placeholder="Cerca in questa vista…" value={q} onChange={e => setQ(e.target.value)} />
            <span className="kbd">⌘F</span>
          </div>
          <button className="btn-primary" onClick={onNew}><Icon name="plus" size={12} /> Nuovo</button>
        </div>

        <div className="list-toolbar" style={{ justifyContent: 'space-between' }}>
          <div className="filter-chips">
            <button className="btn-ghost" onClick={onToggleDensity} title="Cambia densità">
              <Icon name={density === 'preview' ? 'panel-rt' : density === 'comfy' ? 'sidebar' : 'sidebar-right'} size={12} />
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11 }}>{density === 'preview' ? 'Anteprima' : density === 'comfy' ? 'Comoda' : 'Compatta'}</span>
            </button>
            {filterChips.map((c, i) => (
              <span key={i} className={`chip ${c.kind || ''}`}>
                {c.label}
                <span className="x" onClick={() => onClearFilter(c.id)}><Icon name="x" size={9} /></span>
              </span>
            ))}
          </div>
          <button className="btn-ghost" onClick={onSort} title="Ordina">
            <Icon name="sort" size={13} />
            <span style={{ fontSize: 12, fontWeight: 500 }}>{sort}</span>
            <Icon name="chevron-down" size={11} />
          </button>
        </div>
      </div>

      <div className="list-scroll">
        {filtered.map(p => (
          <PromptCard key={p.id} prompt={p} active={activeId === p.id} density={density} previewLines={previewLines} onClick={() => onSelect(p.id)} onToggleFav={onToggleFav} />
        ))}
        {filtered.length === 0 && (
          <div style={{ padding: '40px 20px', textAlign: 'center', color: 'var(--text-subtle)', fontSize: 13 }}>
            <div style={{ marginBottom: 8 }}>Nessun prompt corrispondente</div>
            <button className="btn-secondary" onClick={onNew}><Icon name="plus" size={12} /> Crea nuovo</button>
          </div>
        )}
      </div>
    </section>
  );
}

// ---------- Detail tabs ----------
function DetailTabs({ tab, onTab, lintCount, goldenCount, varsCount, historyCount }) {
  const tabs = [
    { id: 'editor', label: 'Editor', icon: 'edit' },
    { id: 'anteprima', label: 'Anteprima', icon: 'eye' },
    { id: 'diagnosi', label: 'Diagnosi', icon: 'alert-triangle', badge: lintCount, badgeKind: lintCount > 0 ? 'warn' : null },
    { id: 'test', label: 'Test golden', icon: 'flask', badge: goldenCount },
    { id: 'cronologia', label: 'Cronologia', icon: 'history', badge: historyCount },
    { id: 'import', label: 'Import & Var.', icon: 'fork', badge: varsCount }
  ];
  return (
    <div className="detail-tabs">
      {tabs.map(t => (
        <button key={t.id} className={`tab ${tab === t.id ? 'active' : ''}`} onClick={() => onTab(t.id)}>
          <Icon name={t.icon} size={13} />
          {t.label}
          {t.badge != null && t.badge > 0 && <span className={`badge ${t.badgeKind || ''}`}>{t.badge}</span>}
        </button>
      ))}
    </div>
  );
}

// ---------- Mock CodeMirror editor ----------
function MockEditor({ body, dirty, onChange }) {
  const lines = body.split('\n');
  const renderLine = (line, idx) => {
    // tokens: {{import "x"}}, {{var}}
    const parts = [];
    let rest = line;
    let key = 0;
    const re = /(\{\{import\s+"[^"]+"\}\})|(\{\{[a-zA-Z_][\w]*\}\})/g;
    let last = 0; let m;
    while ((m = re.exec(line)) !== null) {
      if (m.index > last) parts.push(<span key={key++}>{line.slice(last, m.index)}</span>);
      if (m[1]) parts.push(<span key={key++} className="import-token">{m[1]}</span>);
      else parts.push(<span key={key++} className="placeholder-token">{m[2]}</span>);
      last = m.index + m[0].length;
    }
    if (last < line.length) parts.push(<span key={key++}>{line.slice(last)}</span>);

    const isLintLine = idx === 4;
    return (
      <div key={idx} className={`cm-line ${idx === 0 ? 'active' : ''}`}>
        {isLintLine ? <span className="lint-warn">{parts.length ? parts : line || ' '}</span> : (parts.length ? parts : line || ' ')}
      </div>
    );
  };

  return (
    <div className="editor-scroll">
      <div className="cm-editor">
        <div className="cm-gutter">
          {lines.map((_, i) => <div key={i}>{i + 1}</div>)}
        </div>
        <div className="cm-content" contentEditable suppressContentEditableWarning onInput={(e) => onChange?.(e.currentTarget.innerText)}>
          {lines.map(renderLine)}
        </div>
      </div>
    </div>
  );
}

// ---------- Right rail (metadata) ----------
function RightRail({ prompt, onUpdate }) {
  return (
    <aside className="right-rail">
      <div className="rail-section">
        <div className="rail-h">Metadati</div>
        <div className="field">
          <label className="field-label">Visibilità</label>
          <div className={`segmented ${prompt.visibility}`}>
            <button className={prompt.visibility === 'private' ? 'on' : ''} onClick={() => onUpdate({ visibility: 'private' })}>
              <Icon name="lock" size={11} /> Privato
            </button>
            <button className={prompt.visibility === 'team' ? 'on' : ''} onClick={() => onUpdate({ visibility: 'team' })}>
              <Icon name="users" size={11} /> Team
            </button>
          </div>
        </div>
        <div className="field">
          <label className="field-label">Modello target</label>
          <select className="field-select" value={prompt.target} onChange={e => onUpdate({ target: e.target.value })}>
            {window.TARGETS.map(t => <option key={t}>{t}</option>)}
          </select>
        </div>
        <div className="field">
          <label className="field-label">Cartella</label>
          <select className="field-select" value={prompt.folder} onChange={e => onUpdate({ folder: e.target.value })}>
            <option>/</option>
            <option>Ruoli</option>
            <option>Ruoli/Developer</option>
            <option>Ruoli/Content</option>
            <option>Email</option>
            <option>Sandbox</option>
          </select>
        </div>
        <div className="field">
          <label className="field-label">Tag</label>
          <div className="tag-picker">
            {prompt.tags.map(t => {
              const tag = window.TAGS.find(x => x.id === t);
              return <span key={t} className="tag-pill"><span className="dot" style={{ background: tag?.color || 'var(--text-muted)' }} />{t}<span className="x"><Icon name="x" size={9} /></span></span>;
            })}
            <input placeholder={prompt.tags.length ? '+' : 'Aggiungi tag…'} />
          </div>
        </div>
      </div>

      <div className="rail-section">
        <div className="rail-h">Segnaposti rilevati <span className="count">{(prompt.placeholders||[]).length}</span></div>
        {(prompt.placeholders||[]).length === 0 && <div style={{ fontSize: 12, color: 'var(--text-subtle)' }}>Nessun <code style={{fontFamily:'var(--font-mono)'}}>{`{{var}}`}</code> nel body.</div>}
        {(prompt.placeholders||[]).map(p => (
          <div className="var-row" key={p.name}>
            <span className="name">{p.name}</span>
            <span className="type">{p.type}{p.default ? ` · default: ${p.default}` : ''}</span>
            <span className="req" title={p.required ? 'Obbligatorio' : 'Opzionale'}>{p.required ? '*' : ''}</span>
          </div>
        ))}
      </div>

      {prompt.imports && prompt.imports.length > 0 && (
        <div className="rail-section">
          <div className="rail-h">Import composti <span className="count">{prompt.imports.length}</span></div>
          {prompt.imports.map(i => (
            <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '4px 0', fontSize: 12 }}>
              <Icon name="fork" size={11} />
              <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--info)' }}>{i}</span>
            </div>
          ))}
        </div>
      )}

      {prompt.variants && prompt.variants.length > 0 && (
        <div className="rail-section">
          <div className="rail-h">Varianti A/B <span className="count">{prompt.variants.length}</span></div>
          <div className="variants-row">
            {prompt.variants.map(v => (
              <span key={v} className={`variant-pill ${v === prompt.isVariant ? 'active' : ''}`}>
                <span className="label">{v}</span>
                <span className="sub">{v === prompt.isVariant ? '· corrente' : ''}</span>
              </span>
            ))}
            <button className="btn-ghost" style={{ height: 24 }}><Icon name="plus" size={11} /> Variante</button>
          </div>
          <button className="btn-ghost" style={{ marginTop: 6, height: 24, padding: '0 6px' }}><Icon name="split" size={11} /> Confronta tutte</button>
        </div>
      )}
    </aside>
  );
}

window.TitleBar = TitleBar;
window.Sidebar = Sidebar;
window.ListPane = ListPane;
window.DetailTabs = DetailTabs;
window.MockEditor = MockEditor;
window.RightRail = RightRail;
window.NavItem = NavItem;
window.NavGroup = NavGroup;
