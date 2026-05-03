# Todo Fase 5 — Ecosistema Enterprise

> **Deliverable finale**: tag release `v1.0.0` (uscita da beta).

## Direzione generale del progetto

Prompt a Porter è una libreria locale-first per prompt AI. Tutte le scelte tecniche seguono tre vincoli non negoziabili:

1. **I dati restano sull'utente.** Vault cifrato locale, feature cloud opt-in, niente telemetria.
2. **Niente lock-in.** Formati aperti (Markdown, JSON), licenza AGPL 3.0, export sempre disponibile, schema dati documentato.
3. **Integrazione via standard.** MCP, OIDC, OpenAPI 3.1, Native Messaging — niente API proprietarie chiuse.

Il progetto attraversa 5 fasi: dall'app standalone (Fase 1, chiusa) alle fondamenta solide e integrabili (Fase 2), all'intelligenza assistiva tutta locale (Fase 3), ai workflow avanzati con qualità misurabile (Fase 4), all'ecosistema enterprise opt-in (Fase 5, questa → v1.0.0).

## Direzione di Fase 5

PaP esce da beta. Da prodotto a piattaforma integrabile. Le feature qui sono tutte **domanda-driven**: entrano se e solo se c'è un caso d'uso reale (cliente pagante, ambiente regolato, integrazione critica). E2E encryption e SSO non sono speculative: si fanno se hai cliente target che le richiede. Browser extension e web app sono nella stessa categoria — costose da mantenere, valide solo con utenti che le usano davvero.

**Il rischio principale di Fase 5 è la sovra-ingegnerizzazione preventiva.** Ogni step di questa fase deve avere risposta affermativa alle domande:
- C'è un utente o cliente reale che lo chiede?
- Il costo di mantenimento è giustificato dal volume d'uso atteso?
- Esiste un'alternativa standard più semplice che copre l'80% del caso?

Se le risposte sono "no", lo step viene rinviato a 1.x.

### Filosofia di Fase

> "Quando Ulisse torna a Itaca, riconosce la sua casa ma sa anche che il mondo intorno è cambiato. Non basta più la sala del trono: serve una rete di alleanze."

In Fase 5 PaP smette di essere "la mia libreria di prompt" e inizia a essere "il nodo della mia infrastruttura AI". Le decisioni qui hanno **impatto architetturale** sui dati esistenti — la cifratura E2E in particolare richiede attenzione perché modifica il modo in cui server e client si scambiano informazioni.

---

## Step 0 — Prerequisiti

- [ ] Fase 4 chiusa: `v0.4.0` taggata, varianti + approval + ACL cartelle + regression testing funzionanti
- [ ] Decisione strategica E2E: chi è l'audience target? (workspace ad alta sensibilità — legale, sanità, difesa, finanza regolata)
- [ ] Decisione strategica web/extension: c'è domanda concreta da utenti reali? Se no → skip e rimanda a 1.x
- [ ] Audit completo dipendenze: tutte compatibili AGPL 3.0
- [ ] Cambio licenza GPL 2.0 → AGPL 3.0 già completato in Fase 2 (verificare consistenza in tutti i file e nel sito web se presente)
- [ ] Crea branch `fase-5` da `main`

---

## Scope Feature Fase 5

| # | Step | Modulo | Condizione di esecuzione |
|---|------|--------|--------------------------|
| 1 | Commenti + reazioni sui prompt | client + server | Solo se utenti team chiedono |
| 2 | OIDC + SAML 2.0 (Enterprise SSO) | server | Solo se cliente enterprise lo richiede |
| 3 | Webhook system | server | Sempre (basso costo, alto valore) |
| 4 | API pubblica documentata + token scopati | server | Sempre (consolidamento) |
| 5 | E2E encryption per workspace ad alta sensibilità | client + server | Solo se cliente target esiste |
| 6 | Web app (subset funzionalità client) | nuovo modulo `apps/web` | Solo se domanda concreta |
| 7 | Browser extension Manifest V3 + Native Messaging | nuovi moduli | Solo se domanda concreta |
| 8 | Sync Git per prompt CI/CD | server | Sempre (basso costo) |

---

## Step 1 — Commenti + reazioni sui prompt

> **Condizione**: solo se ci sono utenti team che chiedono questa feature. Per workspace personali è dead weight.

Estensione del server di sync. Backend prima, UI poi.

- [ ] Nuova tabella server `Comments`: `Id, PromptId, AuthorUserId, ParentCommentId (NULL per root), Body, Reactions (JSON), CreatedAt, UpdatedAt, DeletedAt`
- [ ] Endpoint `POST /prompts/{id}/comments` (crea), `GET /prompts/{id}/comments` (lista thread), `PATCH /comments/{id}` (edit), `DELETE /comments/{id}` (soft delete), `POST /comments/{id}/reactions` (toggle reaction)
- [ ] WebSocket: nuovo channel `comments:{promptId}` per push real-time
- [ ] **Permessi**: tutti i ruoli (Admin/Editor/User) possono commentare e reagire; solo l'autore o Admin possono editare/cancellare
- [ ] **Mention**: parser `@nomeutente` nel body, notifica via WebSocket all'utente menzionato
- [ ] Rate limit: max 10 commenti/min per utente per evitare flood
- [ ] **Solo per workspace team** (i commenti su prompt privati non hanno senso)
- [ ] **UI**: nuovo componente `CommentThread.svelte` nel pannello dettaglio Libreria
- [ ] Editor commento con supporto Markdown base (riusa CodeMirror 6 in modalità minimal)
- [ ] Mention picker con autocomplete utenti workspace
- [ ] Reazioni rapide (4 emoji minimal: funziona, non funziona, idea, domanda)
- [ ] Indicatore "N commenti" nella card della lista
- [ ] Notifica in-app per mention (toast) + dot rosso su tray icon
- [ ] **i18n**: tutte le stringhe in `it.json` e `en.json`

## Step 2 — OIDC / SAML (Enterprise SSO)

> **Condizione**: solo se cliente enterprise lo richiede. È feature da B2B, non da utente individuale.

Per workspace aziendali che richiedono integrazione con Identity Provider esistenti (Entra ID, Okta, Keycloak).

- [ ] Server: integrazione **OIDC** via libreria `coreos/go-oidc` + `golang.org/x/oauth2`
- [ ] Server: integrazione **SAML 2.0** via `crewjam/saml`
- [ ] Configurazione per workspace: admin imposta IdP (Issuer, JWKS URL / IdP metadata XML, mapping claim → ruolo)
- [ ] **Provisioning automatico**: utente che fa login via SSO la prima volta viene creato automaticamente con ruolo default `User` (configurabile)
- [ ] **JIT (Just-In-Time)**: claim → ruolo mapping (es. group `prompt-admins` → ruolo Admin)
- [ ] **Logout SLO**: signal di logout dall'IdP termina sessione PaP
- [ ] **Coesistenza**: SSO opzionale, l'auth password locale resta per workspace senza IdP
- [ ] **Test**: setup Keycloak self-hosted come IdP di test
- [ ] **UI**: pulsante "Login con SSO" in schermata Login (visibile se workspace ha IdP configurato)

## Step 3 — Webhook system

> **Condizione**: sempre. Costo basso, valore alto per integrazioni esterne (n8n, Zapier, monitoring).

Permettere a sistemi esterni di reagire a eventi PaP.

- [ ] Eventi disponibili: `prompt.created`, `prompt.updated`, `prompt.deleted`, `prompt.approved`, `prompt.rejected`, `prompt.golden_failed` (regression testing in Fase 4), `workspace.member_added`, `workspace.member_removed`, `comment.created` (se Step 1 attivo)
- [ ] **UI**: in Impostazioni > Integrazioni, sezione "Webhook" con CRUD endpoint
- [ ] Configurazione per webhook: URL, eventi sottoscritti, header custom, secret (HMAC SHA256 firma payload)
- [ ] Payload JSON standard:
  ```json
  {
    "event": "prompt.created",
    "occurred_at": "2026-...",
    "workspace_id": "...",
    "actor_user_id": "...",
    "data": { ... }
  }
  ```
- [ ] Header `X-PaP-Signature` con HMAC per verifica
- [ ] **Retry**: in caso di errore HTTP non-2xx, retry esponenziale (fino a 5 tentativi in 24h)
- [ ] **Dead letter queue**: webhook falliti dopo retry vanno in lista visibile all'admin per riprova manuale
- [ ] **Privacy**: per workspace E2E, payload include solo metadati (no body cifrato)
- [ ] Test integrazione con n8n / Zapier / endpoint custom

## Step 4 — API pubblica documentata

> **Condizione**: sempre. Consolida l'API server già esistente per uso esterno e CLI/MCP.

Sistematizzare l'API server per uso esterno.

- [ ] **OpenAPI 3.1** spec generata da codice Go (con `kin-openapi` o simile)
- [ ] Endpoint pubblicati sotto `/api/v1/...` (versioning esplicito)
- [ ] Auth: **API Token** generabili in Impostazioni > API Tokens, scopati a workspace + permessi (read-only / read-write)
- [ ] Rate limit: configurabile per token, default 1000 req/h
- [ ] **Documentazione**: pagina `/docs` servita dal server, con Swagger UI o Redoc
- [ ] **Esempi**: `docs/api-esempi.md` con cURL e snippet Python/Node/Go
- [ ] **Tools client**: piccola libreria Python `pap-client` (≥ wrapper minimal su API per uso scripting)

## Step 5 — End-to-End Encryption (E2E)

> **Condizione**: solo se cliente target esiste (legale, sanità, difesa, finanza regolata). Richiede review crittografica esterna prima di andare in produzione.

**Modello**: ogni workspace E2E ha una **Workspace Master Key** (WMK) a 256 bit, mai conosciuta dal server. Ogni utente del workspace ha la WMK cifrata con la propria **User Key** (derivata da password + chiavi asimmetriche per scambio).

**Trade-off espliciti** che vanno spiegati all'utente all'attivazione:
- ❌ Ricerca semantica server-side disabilitata (server non vede il contenuto)
- ❌ Recupero password = perdita irrecuperabile del workspace (no backdoor)
- ❌ Web app limitata: serve sblocco password ogni sessione (no SSO seamless)
- ❌ Webhook payload include solo metadati, non body
- ✅ Server compromesso ≠ prompt compromessi
- ✅ Insider del provider hosting non può leggere i prompt
- ✅ Compliance ARERA / GDPR / NIS2 facilitata per workspace regolati

**Crypto stack**:
- Cifratura simmetrica: **XChaCha20-Poly1305** (audited, costante in tempo)
- Derivation password → User Key: **Argon2id** (m=64MB, t=3, p=4)
- Scambio chiavi: **X25519** (curve Diffie-Hellman moderna)
- Firme: **Ed25519**

**Implementazione**:

- [ ] Nuove tabelle server (i campi cifrati sono base64 di blob cifrati):
  ```sql
  ALTER TABLE Workspaces ADD COLUMN IsE2E INTEGER NOT NULL DEFAULT 0;
  ALTER TABLE Workspaces ADD COLUMN PublicKey TEXT;       -- workspace public key

  CREATE TABLE WorkspaceMembers (
      WorkspaceId         TEXT NOT NULL,
      UserId              TEXT NOT NULL,
      EncryptedWMK        TEXT NOT NULL,  -- WMK cifrata con UserKey via X25519
      AddedByUserId       TEXT NOT NULL,
      AddedAt             TEXT NOT NULL,
      PRIMARY KEY (WorkspaceId, UserId)
  );
  ```
- [ ] Nei prompt: campi `Title`, `Description`, `Body`, `Tags` cifrati con WMK; metadati strutturali (`Id`, `WorkspaceId`, `AuthorUserId`, timestamp) restano in chiaro
- [ ] **Onboarding**: in fase di creazione workspace, opzione "Crittografia end-to-end" con disclaimer chiaro sui trade-off
- [ ] **Aggiunta membro** a workspace E2E: rituale di scambio chiavi
  1. Admin invita utente (genera invito con UserId provvisorio)
  2. Utente accetta da client desktop, genera coppia X25519
  3. Pubblica chiave pubblica al server
  4. Admin riceve notifica, cifra WMK con chiave pubblica utente, invia
  5. Utente riceve EncryptedWMK, può decifrare con propria private key
- [ ] **Rotazione chiavi**: comando admin per cambiare WMK (re-cifra tutto, distribuisce nuova WMK ai membri attivi)
- [ ] **Revoca utente**: rimuovendo da workspace, nuova WMK + re-cifratura (utente revocato non può più decifrare cose nuove, ma i suoi backup locali restano leggibili — è un trade-off accettato)
- [ ] **UI client**: badge prominente per workspace E2E, eventuale icona lucchetto su ogni prompt
- [ ] **Disabilita feature incompatibili** quando E2E attivo: ricerca semantica server-side, statistiche aggregate server-side, web app SSR
- [ ] **Review esterna**: prima di GA, audit del flusso crittografico da terza parte (è la feature ad alto rischio)

## Step 6 — Web app

> **Condizione**: solo se utenti reali la chiedono. Mantenere una seconda UI è caro per sempre. Se il MCP server (Fase 2) e il CLI (Fase 2) coprono già l'80% del caso d'uso "accesso non-desktop", la web app potrebbe essere skippata o ridotta a un client read-only minimo.

Nuovo modulo `apps/web/`. Subset di funzionalità del client desktop, **read-heavy**.

**Stack proposto**:
- **Pure SPA Svelte 5** (no SvelteKit), riuso CSS variables/tokens del client
- Hosting: SPA buildata + servita dallo stesso binario Go (embed con `embed.FS`)

**Funzionalità incluse**:
- [ ] Login (riusa stessi endpoint `/auth/login`)
- [ ] Libreria read-only con tutte le viste (sidebar + lista + dettaglio + cartelle)
- [ ] Renderer/Compilatore funzionante (form + copy)
- [ ] Commenti (read + write, se Step 1 attivo)
- [ ] Ricerca via API server (`/api/v1/search`)
- [ ] **Differenza chiave**: niente vault locale, niente cifratura at-rest sul client web — la web app è **read-only sui prompt cifrati** che il server custodisce in chiaro per i workspace team

**Funzionalità escluse**:
- ❌ Creazione/modifica prompt (rimando a desktop con CTA "Apri in Prompt a Porter")
- ❌ Impostazioni vault, hotkey, tema avanzato
- ❌ Tray icon, command palette globale
- ❌ Workspace E2E (incompatibile con accesso web)

- [ ] Embedding statico nel server: `go:embed dist/web/*` → servito da `/`
- [ ] CSP stretta, nessuna risorsa esterna
- [ ] Build CI: `pnpm -F @pap/web build` → output in `apps/server/internal/web/dist/`
- [ ] Test E2E con Playwright sui flussi principali

## Step 7 — Browser extension Manifest V3 + Native Messaging

> **Condizione**: solo se utenti reali la chiedono **e** sono disposti ad accettare che gli adapter per i singoli siti AI possano rompersi periodicamente. Costo manutenzione alto e ricorrente.
>
> **Alternativa raccomandata**: il MCP server (Fase 2) copre l'integrazione AI senza il problema dei site adapter. Valutare se la browser extension è davvero necessaria o se MCP basta.

Nuovo modulo `apps/extension/`. Compatibilità: Chrome 120+, Edge 120+, Firefox 121+.

- [ ] Scaffolding Manifest V3 con build Vite multi-target (Chrome/Firefox build separate)
- [ ] UI popup minimal: search → lista risultati → seleziona prompt → form segnaposti → "Inietta nella chat" o "Copia"
- [ ] Service worker: gestisce messaging, storage, fetch verso desktop client (Native Messaging) o server (fallback)
- [ ] Content script: site adapter per claude.ai, chatgpt.com, gemini.google.com, copilot.microsoft.com
- [ ] **Adapter pattern**: ogni adapter implementa `detectInputElement()`, `insertText(text)`, `attachUI()`
- [ ] **Resilienza**: ogni adapter ha selectors fallback multipli, e si auto-disabilita pulito se rileva struttura cambiata
- [ ] Permission minime: `activeTab`, `nativeMessaging`, host permissions per i domini AI
- [ ] **Privacy**: nessuna telemetria, nessun analytics, nessun fetch di terze parti
- [ ] Pubblicazione store rimandata a fine Fase 5 (prima validazione interna)

**Native Messaging Host** (bridge extension ↔ desktop):

- [ ] Lato client desktop (Rust): nuovo binario companion `pap-native-host` (oppure subcommand di `pap` con flag `--native-host`)
- [ ] Protocollo Native Messaging standard: stdin/stdout con messaggi JSON length-prefixed
- [ ] Manifest registrato sul SO al primo avvio del desktop client (con consenso utente)
- [ ] **Sicurezza**: solo extension con `extension_id` allowlistato può connettersi
- [ ] Comandi supportati: `list_prompts`, `search`, `render`, `get_recent`, `health`
- [ ] Fallback HTTP locale (es. `localhost:11811` con token random rigenerato a ogni avvio)
- [ ] Test integrazione: extension → host → vault → response, su tutti e tre gli OS

**Avvertenza esplicita**: gli adapter di siti esterni (claude.ai, chatgpt.com, ecc.) **si rompono ogni volta che i vendor cambiano UI**. È un costo di manutenzione perpetuo. Valutare seriamente se vale la pena rispetto al MCP server.

## Step 8 — Sync Git per prompt CI/CD

> **Condizione**: sempre. Costo basso, valore alto per team che vogliono versionare prompt critici insieme al codice.

Per team che vogliono versionare i prompt critici **anche** in Git accanto al codice.

- [ ] Modalità "Sync con repo Git": per workspace, configurare repo remoto (GitHub/GitLab/Gitea) + branch
- [ ] Periodicamente (o on-demand) il server fa export Markdown del workspace e committa al repo
- [ ] Direzione opposta: webhook da GitHub/GitLab triggera reimport in caso di modifiche dirette al repo
- [ ] **Conflict resolution**: timestamp + autore preservati, `merge` documentato in `docs/git-sync.md`
- [ ] **Use case**: prompt usati in pipeline CI/CD (test, code review automation) versionati insieme al codice del progetto
- [ ] **Integrazione regression testing**: il pipeline CI può eseguire `pap test` (Fase 4) sui prompt e bloccare il merge se le golden run falliscono

## Step 9 — Quality gate Fase 5

- [ ] Test coverage ≥ 80% (alziamo l'asticella per release 1.0)
- [ ] **Penetration test** lato server e client: SAST con `semgrep`, DAST con `zap`, audit dipendenze finale
- [ ] **Verifica E2E crypto**: review esterna del flusso crittografico (idealmente da terza parte) — è la feature ad alto rischio
- [ ] Test E2E completi sulle feature attive (SSO, MCP, webhook, CLI, API, eventuale web/extension)
- [ ] Performance: stress test server con 100 client connessi, 10k prompt per workspace, ricerca P95 < 200ms
- [ ] **Documentazione utente** completa: manuale, FAQ, tutorial video (opzionali)
- [ ] **Localization**: italiano + inglese completi, ulteriori lingue valutabili in 1.x

## Step 10 — Documentazione finale 1.0 e release

- [ ] **Manuale utente** completo in `docs/manuale-utente.md` (italiano)
- [ ] **Architettura completa** aggiornata con tutti i moduli
- [ ] `docs/sicurezza.md` con threat model dettagliato e modelli di trust
- [ ] `docs/sso-setup.md` con esempi Entra ID, Okta, Keycloak (se Step 2 attivo)
- [ ] `docs/api-pubblica.md` (riferimento OpenAPI)
- [ ] `docs/migrazione-da-altri-tool.md` (Notion, Obsidian, file Markdown sparsi)
- [ ] **Sito web pubblico statico** (opzionale): landing + docs (Astro / Hugo / VitePress)
- [ ] Verifica finale licenza AGPL 3.0 ovunque (header sorgenti, README, package metadata)
- [ ] **Changelog completo dalla v0.1.0 alla v1.0.0**
- [ ] **Tag `v1.0.0`** con release notes celebrative
- [ ] **Annuncio pubblico** (Hacker News, Lobsters, Reddit r/selfhosted, Mastodon)
- [ ] Setup repo per ricevere contributi (CONTRIBUTING.md DCO, CODE_OF_CONDUCT.md, issue templates, PR template)

---

## Decisioni discrezionali ad alta posta in gioco

1. **Quali Step "domanda-driven" eseguiamo davvero?** Step 1 (commenti), Step 2 (SSO), Step 5 (E2E), Step 6 (web app), Step 7 (extension) sono tutti condizionati a richiesta utente reale. Il rischio è costruire infrastruttura mai usata. **Criterio raccomandato**: ogni step richiede almeno 1 utente esterno che lo abbia chiesto esplicitamente prima di partire.
2. **MCP server vs Browser extension**: il MCP server (Fase 2) copre già il caso "voglio usare i miei prompt da Claude/Cursor". La browser extension copre "voglio usarli su chatgpt.com/gemini.google.com" — ambienti dove MCP non arriva. Vale i ~2-3 mesi di lavoro + manutenzione perpetua degli adapter? Decisione finale qui.
3. **Site web pubblico**: investimento marketing reale o sticker su GitHub readme? Per FOSS minimal è opzionale, per adoption serio aiuta. Non bloccante per 1.0.
4. **Modello distribuzione finale**: solo self-hosted free (AGPL) + supporto pagante eventuale, o anche managed cloud "PaP Cloud"? AGPL ti tutela in entrambi i casi.

---

## Cosa NON è in scope di Fase 5 (eventuale Fase 6+)

Cose che potrebbero arrivare in versioni 1.x successive ma non in 1.0:

- ❌ Mobile app (iOS/Android) — il design pensato è desktop-first, mobile richiede UX dedicata
- ❌ Voice input / Whisper integration
- ❌ Multi-modal prompts (immagini allegate ai prompt)
- ❌ Marketplace pubblico di prompt condivisi tra workspace
- ❌ Template gallery community
- ❌ AI-assisted prompt rewriting (chiedere a un modello di migliorare il tuo prompt)
- ❌ Federation tra istanze PaP (ActivityPub-style)

Sono tutte cose interessanti ma fuori scope per la 1.0.

---

## Riferimenti

- Fase 4 (precedente): `docs/todo-fase-4.md`
- MCP spec: https://modelcontextprotocol.io/
- OWASP threat modeling: https://owasp.org/www-community/Threat_Modeling
- AGPL 3.0: https://www.gnu.org/licenses/agpl-3.0.html
- Crypto reference (XChaCha20-Poly1305 + X25519): https://nacl.cr.yp.to/
