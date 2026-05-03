# Todo Fase 5 — Ecosistema

> **Obiettivo**: aprire Prompt a Porter al mondo. A questo punto il prodotto è maturo come strumento standalone (Fasi 1-4); ora diventa **piattaforma**: integrabile con altri strumenti via MCP/API/webhook, integrabile in contesti enterprise via SSO, e con cifratura end-to-end per i casi d'uso più sensibili.
>
> **Deliverable finale**: tag release `v1.0.0` (uscita da beta).

---

## Filosofia di Fase

> "Quando Ulisse torna a Itaca, riconosce la sua casa ma sa anche che il mondo intorno è cambiato. Non basta più la sala del trono: serve una rete di alleanze."

In Fase 5 PaP smette di essere "la mia libreria di prompt" e inizia a essere "il nodo della mia infrastruttura AI". Le decisioni qui hanno **impatto architetturale** sui dati esistenti — la cifratura E2E in particolare richiede attenzione perché modifica il modo in cui server e client si scambiano informazioni.

---

## Prerequisiti (Step 0)

- [ ] Fase 4 chiusa: `v0.4.0-fase4` taggata, varianti + approval funzionanti
- [ ] Decisione strategica E2E: chi è l'audience target? (workspace ad alta sensibilità — legale, sanità, difesa)
- [ ] Audit completo dipendenze: tutte compatibili GPL 2.0
- [ ] Crea branch `fase-5` da `main`

---

## Scope Feature Fase 5

| # | Feature | Modulo |
|---|---------|--------|
| 1 | E2E encryption per workspace ad alta sensibilità | client + server |
| 2 | MCP server expose libreria a Claude/altri agenti | nuovo modulo `apps/mcp-server` |
| 3 | OIDC / SAML per enterprise | server |
| 4 | Webhook system | server |
| 5 | API pubblica documentata | server |
| 6 | CLI helper `pap` | nuovo modulo `apps/cli` |
| 7 | Integrazione native CI/CD per prompt versioning | server |

---

## Step 1 — End-to-End Encryption (E2E)

**Modello**: ogni workspace E2E ha una **Workspace Master Key** (WMK) a 256 bit, mai conosciuta dal server. Ogni utente del workspace ha la WMK cifrata con la propria **User Key** (derivata da password + chiavi asimmetriche per scambio).

**Trade-off espliciti** che vanno spiegati all'utente all'attivazione:
- ❌ Ricerca semantica server-side disabilitata (server non vede il contenuto)
- ❌ Recupero password = perdita irrecuperabile del workspace (no backdoor)
- ❌ Web app limitata: serve sblocco password ogni sessione (no SSO seamless)
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
- [ ] **Onboarding**: in fase di creazione workspace, opzione "🔒 Crittografia end-to-end" con disclaimer chiaro sui trade-off
- [ ] **Aggiunta membro** a workspace E2E: rituale di scambio chiavi
  1. Admin invita utente (genera invito con UserId provvisorio)
  2. Utente accetta da client desktop, genera coppia X25519
  3. Pubblica chiave pubblica al server
  4. Admin riceve notifica, cifra WMK con chiave pubblica utente, invia
  5. Utente riceve EncryptedWMK, può decifrare con propria private key
- [ ] **Rotazione chiavi**: comando admin per cambiare WMK (re-cifra tutto, distribuisce nuova WMK ai membri attivi)
- [ ] **Revoca utente**: rimuovendo da workspace, nuova WMK + re-cifratura (utente revocato non può più decifrare cose nuove, ma i suoi backup locali restano leggibili — è un trade-off accettato)
- [ ] **UI client**: badge 🔒 prominente per workspace E2E, eventuale icona lucchetto su ogni prompt
- [ ] **Disabilita feature incompatibili** quando E2E attivo: ricerca semantica server-side, statistiche aggregate server-side, web app SSR

## Step 2 — MCP Server (Model Context Protocol)

Esporre la libreria PaP a **Claude e altri agenti AI** che parlano MCP. Il vault diventa una "memory" interrogabile da Claude.

Riusa il know-how del MCP framework Bluenergy (governance + scoping). Il design qui è "personal MCP" — niente multi-tenant complesso.

- [ ] Nuovo modulo `apps/mcp-server/` in **Go** (coerente con server di sync) o **TypeScript** (più diffuso ecosistema MCP). **Decisione consigliata**: TypeScript con SDK ufficiale `@modelcontextprotocol/sdk`
- [ ] Trasporti supportati: **stdio** (per uso desktop locale, integrato in Claude Desktop) + **HTTP/SSE** (per accesso remoto)
- [ ] **Tools esposti**:
  - `pap_search(query, limit?, target_model?, tags?)` → ricerca prompt
  - `pap_get(prompt_id)` → dettaglio prompt
  - `pap_list_recent(limit?)` → ultimi usati
  - `pap_render(prompt_id, vars)` → compila template con valori
  - `pap_create_draft(title, body, tags?)` → crea bozza (richiede approvazione manuale via UI desktop)
- [ ] **Resources esposte** (per agentic workflows):
  - `pap://prompt/{id}` → resource del singolo prompt
  - `pap://workspace/{id}/recent` → lista recenti come resource
- [ ] **Prompts MCP** (sì, prompt-of-prompts!):
  - `pap_help_write_prompt` → guida l'AI a scrivere un buon prompt seguendo le regole linting di Fase 3
- [ ] **Auth**: per HTTP/SSE, token bearer scopato a un workspace; per stdio, accesso al vault locale via Native Messaging
- [ ] **Permission gate**: ogni tool call mostra notifica desktop "Claude vuole leggere prompt 'X', consenti?" (configurabile: auto-allow per workspace personali, ask per team)
- [ ] **Limiti**: niente MCP tool che modifica/cancella senza approval esplicita umana
- [ ] **Documentazione**: `docs/mcp-integration.md` con esempi concreti per Claude Desktop, Cursor, etc.

## Step 3 — OIDC / SAML (Enterprise SSO)

Per workspace aziendali che richiedono integrazione con Identity Provider esistenti (Entra ID, Okta, Keycloak).

- [ ] Server: integrazione **OIDC** via libreria `coreos/go-oidc` + `golang.org/x/oauth2`
- [ ] Server: integrazione **SAML 2.0** via `crewjam/saml`
- [ ] Configurazione per workspace: admin imposta IdP (Issuer, JWKS URL / IdP metadata XML, mapping claim → ruolo)
- [ ] **Provisioning automatico**: utente che fa login via SSO la prima volta viene creato automaticamente con ruolo default `User` (configurabile)
- [ ] **JIT (Just-In-Time)**: claim → ruolo mapping (es. group `prompt-admins` → ruolo Admin)
- [ ] **Logout SLO**: signal di logout dall'IdP termina sessione PaP
- [ ] **Coesistenza**: SSO opzionale, l'auth password locale resta per workspace senza IdP
- [ ] **Test**: setup Keycloak self-hosted come IdP di test (Roberto può usare uno già in casa Bluenergy o nel lab)
- [ ] **UI**: pulsante "Login con SSO" in schermata Login (visibile se workspace ha IdP configurato)

## Step 4 — Webhook system

Permettere a sistemi esterni di reagire a eventi PaP.

- [ ] Eventi disponibili: `prompt.created`, `prompt.updated`, `prompt.deleted`, `prompt.approved`, `prompt.rejected`, `workspace.member_added`, `workspace.member_removed`, `comment.created`
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

## Step 5 — API pubblica documentata

Sistematizzare l'API server per uso esterno.

- [ ] **OpenAPI 3.1** spec generata da codice Go (con `kin-openapi` o simile)
- [ ] Endpoint pubblicati sotto `/api/v1/...` (versioning esplicito)
- [ ] Auth: **API Token** generabili in Impostazioni > API Tokens, scopati a workspace + permessi (read-only / read-write)
- [ ] Rate limit: configurabile per token, default 1000 req/h
- [ ] **Documentazione**: pagina `/docs` servita dal server, con Swagger UI o Redoc
- [ ] **Esempi**: `docs/api-esempi.md` con cURL e snippet Python/Node/Go
- [ ] **Tools client**: piccola libreria Python `pap-client` (>= un wrapper minimal su API per uso scripting)

## Step 6 — CLI `pap`

Strumento da terminale per power user e automazioni.

- [ ] Modulo `apps/cli/` in **Go** (single binary multipiattaforma, coerente con server)
- [ ] Comandi:
  - `pap login` — autentica contro server o usa Native Messaging con desktop
  - `pap search "query"` — cerca prompt, output table o JSON
  - `pap get <id>` — mostra prompt dettaglio
  - `pap render <id> --var key=value ...` — compila e stampa
  - `pap render <id> --var-file vars.yaml | xclip` — pipe-friendly
  - `pap new <file.md>` — crea prompt da file Markdown con front-matter
  - `pap export --format=json --workspace=team` — bulk export
  - `pap import file.json` — bulk import
- [ ] **Output formats**: `--format=table` (default) | `json` | `yaml` | `plain`
- [ ] Config in `~/.config/pap/config.toml` (server URL, token attivo, workspace default)
- [ ] **Test**: ogni comando ha test E2E contro server in container

## Step 7 — Versioning prompt nativo CI/CD

Per team che vogliono versionare i prompt critici **anche** in Git accanto al codice.

- [ ] Modalità "Sync con repo Git": per workspace, configurare repo remoto (GitHub/GitLab/Gitea) + branch
- [ ] Periodicamente (o on-demand) il server fa export Markdown del workspace e committa al repo
- [ ] Direzione opposta: webhook da GitHub/GitLab triggera reimport in caso di modifiche dirette al repo
- [ ] **Conflict resolution**: timestamp + autore preservati, `merge` documentato in `docs/git-sync.md`
- [ ] **Use case**: prompt usati in pipeline CI/CD (test, code review automation) versionati insieme al codice del progetto

## Step 8 — Quality gate Fase 5

- [ ] Test coverage ≥ 80% (alziamo l'asticella per release 1.0)
- [ ] **Penetration test** lato server e client: SAST con `semgrep`, DAST con `zap`, audit dipendenze finale
- [ ] **Verifica E2E crypto**: review esterna del flusso crittografico (idealmente da terza parte) — è la feature ad alto rischio
- [ ] Test E2E completi: SSO, MCP, webhook, CLI, API
- [ ] Performance: stress test server con 100 client connessi, 10k prompt per workspace, ricerca P95 < 200ms
- [ ] **Documentazione utente** completa: manuale, FAQ, tutorial video (opzionali)
- [ ] **Localization**: italiano + inglese completi, ulteriori lingue valutabili in 1.x

## Step 9 — Documentazione finale 1.0 e release

- [ ] **Manuale utente** completo in `docs/manuale-utente.md` (italiano)
- [ ] **Architettura completa** aggiornata con tutti i moduli
- [ ] `docs/sicurezza.md` con threat model dettagliato e modelli di trust
- [ ] `docs/mcp-integration.md`
- [ ] `docs/sso-setup.md` con esempi Entra ID, Okta, Keycloak
- [ ] `docs/api-pubblica.md` (riferimento OpenAPI)
- [ ] `docs/cli-reference.md`
- [ ] `docs/migrazione-da-altri-tool.md` (Notion, Obsidian, file Markdown sparsi)
- [ ] **Sito web pubblico statico** (opzionale): `docs/promptaporter.it` con landing + docs (Astro / Hugo / VitePress)
- [ ] **Cambio licenza?** Verifica con autore originale se mantenere GPL 2.0 o passare a AGPL 3.0 per chiudere loophole SaaS
- [ ] **Changelog completo dalla v0.1.0 alla v1.0.0**
- [ ] **Tag `v1.0.0`** con release notes celebrative
- [ ] **Annuncio pubblico** (Hacker News, Lobsters, Reddit r/selfhosted, Mastodon)
- [ ] Setup repo per ricevere contributi (CONTRIBUTING.md, CODE_OF_CONDUCT.md, issue templates, PR template)

---

## Decisioni discrezionali ad alta posta in gioco

1. **E2E è feature opt-in del prodotto base o "Enterprise tier"?** GPL 2.0 implica che tutto è open source, ma puoi avere "self-hosted free" + "managed cloud per chi non vuole hostare" come modello. Per ora resta tutto incluso.
2. **MCP server: stdio + HTTP, o solo uno?** Stdio per Claude Desktop integration; HTTP per agenti remoti. Conviene entrambi.
3. **CLI in Go o in Rust?** Go è coerente con server e basso overhead. Rust riusa codice del client Tauri. Preferenza: **Go** per lock-in tooling minimo.
4. **AGPL 3.0 vs GPL 2.0**: AGPL chiude loophole "uso come servizio cloud senza distribuire codice". Se ti aspetti che terzi facciano managed PaP cloud, AGPL ti tutela. Decisione finale prima della 1.0.
5. **Site web pubblico**: investimento marketing reale o sticker su GitHub readme? Per FOSS minimal è opzionale, per adoption serio aiuta.

---

## Cosa NON è in scope di Fase 5 (eventuale Fase 6+)

Cose che potrebbero arrivare in versioni 1.x successive ma non in 1.0:

- ❌ Mobile app (iOS/Android) — il design pensato è desktop-first, mobile richiede UX dedicata
- ❌ Voice input / Whisper integration
- ❌ Multi-modal prompts (immagini allegate ai prompt)
- ❌ Marketplace pubblico di prompt condivisi tra workspace
- ❌ Template gallery community
- ❌ AI-assisted prompt rewriting (chiedere a Claude di migliorare il tuo prompt)
- ❌ Federation tra istanze PaP (ActivityPub-style)

Sono tutte cose interessanti ma fuori scope per la 1.0.

---

## Riferimenti

- Fase 4 (precedente): `docs/todo-fase-4.md`
- MCP spec: https://modelcontextprotocol.io/
- OWASP threat modeling: https://owasp.org/www-community/Threat_Modeling
- AGPL vs GPL: https://www.gnu.org/licenses/license-recommendations.html
