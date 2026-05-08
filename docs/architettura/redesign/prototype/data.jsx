// Mock data for the redesign prototype

const PROMPTS = [
  {
    id: 'p1',
    title: 'Esperto devel Python (B) (fork)',
    desc: 'Variante B forkata — focus su typing rigoroso e Pydantic v2.',
    tags: ['ruolo', 'python'],
    visibility: 'private',
    target: 'Claude Sonnet',
    folder: 'Ruoli',
    favorite: true,
    useCount: 47,
    forkOf: 'Esperto devel Python (B)',
    isVariant: 'B',
    variants: ['A', 'B', 'C'],
    updated: 'ieri · 17:42',
    updatedRaw: '2026-05-07T17:42',
    body: `Sei un esperto sviluppatore Python con 30 anni di esperienza.

Specializzato in: {{specializzazione}}

Quando rispondi:
1. Usa type hints completi (PEP 484, PEP 695)
2. Preferisci Pydantic v2 per validation
3. Scrivi docstring stile {{stile_docstring}}
4. Suggerisci sempre test pytest

{{import "convenzioni/pep8-strict"}}

Contesto extra:
{{contesto}}`,
    placeholders: [
      { name: 'specializzazione', type: 'testo', required: true, default: '' },
      { name: 'stile_docstring', type: 'enum', required: false, default: 'google' },
      { name: 'contesto', type: 'multilinea', required: false, default: '' }
    ],
    imports: ['convenzioni/pep8-strict'],
    lint: [
      { level: 'info', code: 'IMP004', msg: 'Questo prompt è importato da 2 altri prompt', line: null },
      { level: 'warn', code: 'STY001', msg: 'Ripetizione "type hints" in 2 frasi adiacenti', line: 5 }
    ],
    goldens: [
      { id: 'g1', label: 'Type hints completi su funzione asincrona', status: 'pass', last: '14m fa', model: 'claude-sonnet-4.7' },
      { id: 'g2', label: 'Refactor verso Pydantic v2', status: 'pass', last: '14m fa', model: 'claude-sonnet-4.7' },
      { id: 'g3', label: 'Suggerisce pytest fixtures', status: 'fail', last: '14m fa', model: 'claude-sonnet-4.7' }
    ],
    history: [
      { v: 'v3', head: true, author: 'Roberto M.', authorInit: 'RM', team: false, ts: '2026-05-07 17:42', summary: 'Aggiunto import pep8-strict, riformulato punto 2', delta: { add: 4, del: 1 } },
      { v: 'v2', author: 'Roberto M.', authorInit: 'RM', team: false, ts: '2026-05-06 14:08', summary: 'Promossa a variante B con focus typing', delta: { add: 12, del: 3 } },
      { v: 'v1', author: 'Roberto M.', authorInit: 'RM', team: false, ts: '2026-05-04 09:31', summary: 'Creazione iniziale (fork da Esperto devel Python)', delta: { add: 18, del: 0 } }
    ]
  },
  { id: 'p2', title: 'Esperto devel Python (B)', desc: 'Variante B — focus typing rigoroso.', tags: ['ruolo', 'python'], visibility: 'private', target: 'Claude Sonnet', folder: 'Ruoli', useCount: 23, updated: 'ieri · 14:08', body: 'Sei un esperto Python esperto in {{contesto}}.' },
  { id: 'p3', title: 'Esperto devel Python', desc: 'Prompt base ruolo developer Python.', tags: ['ruolo', 'python'], visibility: 'private', target: 'Tutti', folder: 'Ruoli', useCount: 89, updated: 'ieri · 09:31', body: 'Sei uno sviluppatore Python.' },
  { id: 'p4', title: 'Prova com Import', desc: 'Test composizione tramite {{import}}.', tags: ['test'], visibility: 'private', target: 'Tutti', folder: '/', useCount: 4, updated: 'ieri · 11:02', body: '{{import "ruoli/python"}}\n\nLavora su: {{task}}' },
  { id: 'p5', title: 'Prova', desc: 'prompt di prova senza tag', tags: [], visibility: 'private', target: 'Tutti', folder: '/', useCount: 2, updated: 'ieri · 10:18', body: 'Prompt di prova generico.' },
  { id: 'p6', title: 'prova con tag', desc: 'prova con tag', tags: ['test'], visibility: 'private', target: 'Tutti', folder: '/', useCount: 1, updated: 'ieri · 09:55', body: 'Test con tag.' },
  { id: 'p7', title: 'Email cold outreach B2B', desc: 'Template per email di apertura ai clienti enterprise.', tags: ['email', 'sales'], visibility: 'team', target: 'GPT-4', folder: 'Email', useCount: 154, updated: '4g fa', body: 'Scrivi una mail di {{tono}} per {{azienda}}.' },
  { id: 'p8', title: 'Code review checklist', desc: 'Lista standard per code review approfondita.', tags: ['ruolo', 'review'], visibility: 'team', target: 'Claude Opus', folder: 'Ruoli', useCount: 67, updated: '1sett fa', body: 'Esegui una code review di {{linguaggio}} considerando:\n- Sicurezza\n- Performance\n- Leggibilità' }
];

const FOLDERS = [
  { id: 'f-none', name: 'Senza cartella', count: 3, icon: 'none' },
  { id: 'f-ruoli', name: 'Ruoli', count: 3, icon: 'folder', open: true, children: [
    { id: 'f-ruoli-dev', name: 'Developer', count: 2, icon: 'folder' },
    { id: 'f-ruoli-content', name: 'Content', count: 1, icon: 'folder' }
  ] },
  { id: 'f-email', name: 'Email', count: 1, icon: 'folder' },
  { id: 'f-test', name: 'Sandbox', count: 2, icon: 'folder' }
];

const TAGS = [
  { id: 'python', name: 'python', color: 'oklch(0.66 0.13 220)', count: 3 },
  { id: 'ruolo', name: 'ruolo', color: 'oklch(0.70 0.14 30)', count: 4 },
  { id: 'email', name: 'email', color: 'oklch(0.72 0.14 150)', count: 1 },
  { id: 'sales', name: 'sales', color: 'oklch(0.70 0.16 320)', count: 1 },
  { id: 'review', name: 'review', color: 'oklch(0.74 0.14 80)', count: 1 },
  { id: 'test', name: 'test', color: 'oklch(0.60 0.05 270)', count: 2 }
];

const TARGETS = [
  'Tutti', 'Claude Opus', 'Claude Sonnet', 'Claude Haiku',
  'GPT-4', 'GPT-4 Mini', 'Gemini Pro', 'Gemini Flash', 'Llama 3', 'Generico'
];

window.PROMPTS = PROMPTS;
window.FOLDERS = FOLDERS;
window.TAGS = TAGS;
window.TARGETS = TARGETS;
