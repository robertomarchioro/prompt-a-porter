# Blueprint — Guida/Tutorial interattivo in-app

> Stato: **design, no codice** (2026-06-16). Feature di onboarding/help. Pensata per la linea Personale (v1.x): rende il prodotto "completo" sul fronte apprendibilità. Approfondimenti rinviati alla **documentazione online** (sito, vedi `docs/roadmap/website/`).

## 1. Obiettivo

Un utente nuovo apre Prompt a Porter e si trova davanti un'interfaccia a tre pannelli (sidebar viste/cartelle/tag · lista prompt · editor con CodeMirror) ricca di funzioni non ovvie: segnaposti `{{nome}}`, segnaposti globali `{{globale ...}}`, import componibili `{{import "x" with k=v}}`, varianti A/B, rating, golden/test, cestino, linter, ricerca ibrida. La curva di apprendimento è alta.

Vogliamo:

- **Ridurre il time-to-first-value**: l'utente capisce in <2 minuti cosa fa ogni area e come creare/compilare il primo prompt.
- **Insegnare le funzioni avanzate al momento giusto**, senza scommergere subito.
- **Tenere la profondità fuori dall'app**: spiegazioni brevi in-app, dettaglio sul sito (link "Approfondisci").
- Non essere invadenti per chi sa già muoversi.

## 2. La proposta originale (spotlight tour)

> "Tutorial interattivi che evidenziano una parte della schermata e spiegano cosa fa la specifica funzione; gli approfondimenti alla documentazione online."

È un **coachmark/spotlight tour**: un overlay scurisce la schermata, "buca" un riquadro attorno all'elemento attivo e mostra un popover con testo + Avanti/Indietro/Salta.

**Punti di forza**: guidato, mostra esattamente *dove* sta una funzione, alta efficacia sul primo contatto, pattern familiare (lo conoscono tutti).

**Limiti da affrontare** (è qui che la miglioriamo):

- **Fragilità**: i passi sono ancorati a elementi del DOM; ogni refactor UI (ne abbiamo fatto uno grosso, il redesign v0.8) rompe gli ancoraggi se non c'è un contratto stabile.
- **Può risultare paternalistico/forzato** se lungo e obbligatorio.
- **Accessibilità**: molti tour-overlay sono deboli su focus management, tastiera, screen reader — e noi abbiamo investito su WCAG AA (M2).
- **Manutenzione dei testi** + **i18n** (l'app è IT con chiavi i18n).
- **Una sola tipologia di apprendimento** (guarda-e-leggi); alcuni utenti imparano facendo.

## 3. Proposta migliorata — sistema di aiuto a strati

Invece di "un grande tour", un **help system stratificato** dove lo spotlight tour è solo uno dei livelli. Ogni livello è utile da solo e degradano con grazia.

1. **Tour guidato di benvenuto** (breve, 5-7 passi) — parte **dopo** l'OnboardingWizard (setup vault), non lo duplica. Evidenzia solo le 5 aree cardine: viste/sidebar → crea prompt → editor & segnaposti → compila/anteprima → ricerca. Saltabile e **ripetibile**.
2. **Micro-tour contestuali per-feature** (2-3 passi) — si offrono **alla prima apertura** di un'area avanzata (pannello Test/Golden, Varianti, Import componibili, Cestino). Opt-in, non bloccanti. Insegnano "appena prima del bisogno" invece che tutto all'inizio.
3. **Menu "Guida"** persistente (header/menu app) — per: ri-lanciare qualunque tour, aprire la **documentazione online**, vedere le **scorciatoie** (già esistono in `docs/utente/`).
4. **Affordance "?" contestuali** sui pannelli complessi — un piccolo "?" che apre il **deep-link** alla sezione giusta dei docs online (chi salta il tour ha comunque aiuto on-demand).
5. **Checklist "Primi passi"** (opzionale, dismissibile) — pannellino che traccia le prime azioni chiave (crea un prompt · aggiungi un tag · usa un segnaposto · compila · prova un import). Insegna *facendo* e dà senso di progresso.
6. **Documentazione online** (sito) per la profondità — destinazione di tutti i link "Approfondisci"/"?".

Perché stratificato: il tour spotlight da solo è un single point of failure (se lo salti, non impari; se si rompe, niente aiuto). Gli strati 3-4-5 restano utili anche senza il tour e sono molto più robusti.

## 4. Alternative valutate (pro/contro)

| # | Soluzione | Pro | Contro |
|---|-----------|-----|--------|
| **A** | **Spotlight tour** (la proposta) via libreria (driver.js/Shepherd) | Guidato, mostra il "dove", alta efficacia primo contatto, pattern noto | Fragile ai refactor UI, può risultare forzato, lavoro a11y, manutenzione passi |
| **B** | **Tour custom in Svelte** (overlay proprio) | Controllo totale su a11y/i18n/tema, zero dipendenze/licenze, integra coi runes | Da costruire e mantenere; il **posizionamento** (flip/shift/resize/scroll) è la parte difficile |
| **C** | **Hotspot "?" non-lineari** (coachmark on-demand) | Self-paced, poco invadente, robusto ai cambi UI, scopri quando vuoi | Non garantisce che l'utente impari il *flusso*; serve contenuto per elemento |
| **D** | **Empty-state + hint inline** ("Nessun prompt: creane uno con…") | Contestuale, insegna facendo, a11y nativa, zero overlay | Copre solo aree con stato vuoto; non è un "tour" |
| **E** | **Esplorazione vault demo + checklist** (riusa `demo-vault.json`) | Impara su contenuti reali, gamificato/sticky, basso rischio tecnico | Serve tracking stato checklist; non evidenzia gli elementi |
| **F** | **Video/GIF** embedded o linkati | Ricchi, economici da produrre | Non interattivi, costo i18n, invecchiano coi cambi UI, peso |
| **G** | **Solo docs online interattivi** (niente in-app) | Minima complessità nell'app | Context-switch fuori dall'app; gli utenti non li leggono |

Nessuna è esclusiva: la proposta migliorata (§3) **combina A (ridotto) + C + D + E + il sito (G)**, lasciando F come opzione futura sul sito.

## 5. Raccomandazione

**Adottare il sistema a strati (§3)**, con questo ordine di valore/sforzo:

- **Subito ad alto valore / basso sforzo**: strato 3 (menu Guida + link docs), strato 4 (affordance "?"), strato 5 (checklist primi passi), e gli empty-state hint (D). Robusti, a11y-friendly, non fragili.
- **Spotlight tour (A) come ciliegina**: il tour di benvenuto + i micro-tour, costruiti **sopra un contratto di ancoraggio stabile** (vedi §6) così da non essere fragili.

### Libreria vs custom per lo spotlight

- **driver.js** (MIT, ~5 kB, framework-agnostic): highlight + popover, semplice, manutenuto. Va **incapsulato** in un componente Svelte per tema/i18n/a11y. **Path consigliato per il prototipo.**
- **Shepherd.js** (più ricco, usa Floating UI): più funzioni ma più peso/complessità.
- **intro.js**: ⚠️ **licenza dual AGPL/commerciale** — da valutare con attenzione contro la nostra **AGPL-3.0-only** (probabile compatibilità, ma è un vincolo in più). Lo eviterei se driver.js basta.
- **Custom (B)**: se in fase di prototipo driver.js **non regge la barra WCAG AA** (focus trap, tastiera, screen reader, `prefers-reduced-motion`) o il theming non si integra, si costruisce un engine sottile con **`@floating-ui/dom`** per il posizionamento (flip/shift/autoUpdate su scroll/resize — fondamentale perché i pannelli sono ridimensionabili con paneforge).

**Decisione raccomandata**: prototipo con **driver.js incapsulato**; gate di accettazione = WCAG AA + tema + i18n + ripristino posizione su resize. Se fallisce → engine custom su Floating UI. In entrambi i casi il **contratto `data-tour` (§6) è invariato**, quindi la scelta libreria è reversibile.

## 6. Architettura tecnica

### 6.1 Contratto di ancoraggio `data-tour`
La decisione di design più importante per non essere fragili: i passi del tour **non** puntano a selettori CSS/struttura DOM, ma ad attributi stabili.

```html
<button data-tour="crea-prompt"> … </button>
<aside data-tour="sidebar-viste"> … </aside>
```

Una mappa centrale (`tour-steps.ts`) associa `data-tour` id → testo i18n + link doc + opzioni (posizione popover, azione attesa). I refactor UI restano liberi finché preservano gli attributi; un test/lint può verificare che ogni `data-tour` referenziato dai passi esista nel markup.

### 6.2 Engine
- Componente `GuidaTour.svelte` (Svelte 5 runes) che orchestra: stato passo corrente, overlay, popover, navigazione.
- Posizionamento via libreria o `@floating-ui/dom` con `autoUpdate` (riposiziona su scroll/resize — necessario coi pannelli paneforge e il window resize di Tauri).
- `scrollIntoView` morbido sull'elemento target prima di evidenziarlo.
- Passi **condizionali/attesa-azione** opzionali: alcuni passi avanzano solo dopo che l'utente compie l'azione (es. "clicca qui per creare un prompt") → apprendimento attivo.

### 6.3 Persistenza & versioning
- Flag per-tour in **preferenze** (sistema prefs già esistente): `visto`, `completato`, `saltato`, + `versione_tour`.
- **Versioning**: ogni tour ha una versione; un cambio UI importante la incrementa → il tour viene ri-offerto ("Novità: rivedi il tour aggiornato").
- Reset in **Impostazioni → ?/Guida** ("rivedi i tutorial").

### 6.4 i18n & contenuti
- Testi come **chiavi i18n** (coerente col resto dell'app). Brevi (1-2 frasi) per passo; la profondità va al link.
- Contenuto separato dal codice in `tour-steps.ts` + file i18n.

### 6.5 Accessibilità (gate, non opzionale)
- Focus trap sul popover; **ESC** esce; **frecce/Tab** navigano; `aria-live` annuncia il testo del passo; `role="dialog"`/`aria-modal`.
- Rispetto `prefers-reduced-motion` (niente animazioni aggressive).
- L'highlight non deve rendere l'elemento non focalizzabile da screen reader.

### 6.6 Deep-link alla documentazione online
- Ogni passo/“?” ha un `docHref` → URL del sito su un'**ancora specifica** (`/docs/editor#segnaposti-globali`).
- **Offline-first**: l'app è locale; aprire un URL esterno richiede rete. Mitigazioni: (a) i docs utente esistono già in `docs/utente/*.md` (M8) → in mancanza di rete aprire/visualizzare la versione locale; (b) o accettare il "apri nel browser" come comportamento standard. Decisione aperta (§9).

## 7. Integrazione con l'esistente

- **OnboardingWizard**: il tour di benvenuto parte **al termine** dell'onboarding (o viene *offerto* lì con "Fai un giro guidato?"). Niente sovrapposizioni: l'onboarding configura il vault, il tour spiega l'UI.
- **Vault demo** (`docs/demo/demo-vault.json`): la checklist/tour può lavorare su contenuti demo reali (già importabili in onboarding) invece che su un vault vuoto.
- **Docs utente M8** (`docs/utente/`: getting-started, glossario, scorciatoie, troubleshooting, ricette): sono la base testuale; il sito ne è la proiezione online e la destinazione dei link "Approfondisci".
- **Sito** (`docs/roadmap/website/`): prerequisito per i link profondi; finché non c'è, i "?" puntano ai docs locali.
- **Menu/scorciatoie**: integrare un comando "Guida" anche nella **command palette** (Ctrl+Shift+P) e — quando esisterà — nel **menu contestuale** (blueprint `menu-contestuale.md`).

## 8. Fasi di rilascio (incrementali, ognuna rilasciabile)

- **Fase 0 — Fondamenta a basso rischio**: menu "Guida" + link ai docs (locali ora, sito poi) + affordance "?" sui 3-4 pannelli più ostici + empty-state hint. *Valore immediato, zero fragilità.*
- **Fase 1 — Tour di benvenuto**: contratto `data-tour`, engine (driver.js incapsulato), 5-7 passi, persistenza + reset, a11y gate. Offerto post-onboarding.
- **Fase 2 — Micro-tour per-feature**: Test/Golden, Varianti, Import, Cestino — innescati alla prima apertura.
- **Fase 3 — Checklist "Primi passi"** + (opzionale) esplorazione vault demo guidata.
- **Fase 4 — Sito docs online** + deep-link da ogni passo/“?” (dipende dal blueprint website).

## 9. Rischi & domande aperte

- **Fragilità ancoraggi** → mitigata dal contratto `data-tour` + test di esistenza. (rischio: medio→basso)
- **a11y dell'overlay** → gate esplicito; fallback a engine custom se la libreria non regge. (rischio: medio)
- **Offline vs docs online**: apriamo URL esterni (richiede rete) o serviamo i docs locali in-app? **Decisione da prendere.** (Propendo per: link al sito + fallback locale ai `.md` di M8 se offline.)
- **Invadenza**: il tour dev'essere *offerto*, mai forzato; salta-sempre rispettato e ricordato.
- **Manutenzione contenuti i18n** al variare dell'UI → tenere i testi brevi e versionare i tour.
- **Dipendenza dal sito**: le fasi 0-3 NON devono dipendere dal sito (devono funzionare con i docs locali), così la guida è utile da subito.
- **Telemetria**: l'app è privacy-first/local — **niente analytics**. Misuriamo il successo in modo qualitativo (feedback) o con un contatore locale "tour completati" non esfiltrato.

## 10. Sintesi (TL;DR)

La tua idea (spotlight tour) è giusta come **primo contatto**, ma da sola è fragile e a strato unico. La rendiamo robusta e completa trasformandola in un **help system a strati**: menu Guida + "?" contestuali + empty-state (subito, robusti) → tour di benvenuto e micro-tour per-feature (sopra un contratto `data-tour` stabile, driver.js o engine custom) → checklist primi passi → docs online per la profondità. Si rilascia in fasi indipendenti, ognuna utile da sola, e non dipende dal sito per partire.
