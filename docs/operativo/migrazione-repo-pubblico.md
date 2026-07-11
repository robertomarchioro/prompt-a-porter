# Migrazione a doppio repo: dev privato + vetrina pubblica (name swap)

> Checklist operativa — strada A decisa il 2026-07-11.
> Obiettivo: il repo di sviluppo diventa **privato** (`prompt-a-porter-dev`), la
> vetrina diventa **pubblica** ed eredita il nome **`prompt-a-porter`**, così
> tutti gli URL esistenti (endpoint auto-update in primis) restano validi e le
> build di release girano gratis sul repo pubblico.

## Perché il name swap

- L'endpoint updater è cotto dentro le app installate:
  `https://github.com/robertomarchioro/prompt-a-porter/releases/latest/download/latest.json`
  (`apps/client/src-tauri/tauri.conf.json`). Se il nome `prompt-a-porter` non
  resta pubblico, gli utenti esistenti non ricevono più aggiornamenti.
- Il module path Go, i link UI (`docs-links.ts`, `AboutSezione`,
  `ImpostazioniModal`), README/SECURITY, il `base` di VitePress — 69 occorrenze
  in 32 file — restano tutti validi senza toccare una riga.
- GitHub Actions è gratuito e illimitato sui repo pubblici: la release 3 OS
  (macOS ×10 sui privati ≈ 1.500 min fatturati a build) si sposta a costo zero.

## Stato di partenza (verificato 2026-07-11)

| Cosa | Stato |
|---|---|
| `robertomarchioro/prompt-a-porter` (dev) | PUBBLICO, storia completa, release fino a v0.8.35, **Pages ON: sito+landing live su `robertomarchioro.github.io/prompt-a-porter/` dal 2026-07-11** |
| `robertomarchioro/PromptAPorter` (vetrina) | PRIVATO, fermo allo snapshot v0.8.12 (2026-06-04), 0 tag, Pages off |
| Secrets release su dev | `TAURI_SIGNING_PRIVATE_KEY` + `_PASSWORD`, `APPLE_*` ×6 (i `CERTUM_*` NON servono in CI: Authenticode è locale via `sign-release.ps1`) |
| Ultimo tag | v0.8.35 (HEAD di main è 3 commit oltre) |

---

## Fase 0 — Prerequisiti (sul repo dev, prima di toccare GitHub)

- [ ] **0.1 Backup chiavi** (già pendente da giorni): `~/.tauri/apple`
      (certificato + credenziali notarizzazione) e chiave privata Tauri Updater
      (vedi `docs/contribuire/setup-tauri-updater-keys.md`). I secret GitHub
      sono write-only: per replicarli sulla vetrina servono i **valori
      originali**, non c'è copia repo→repo.
- [ ] **0.2 Modifica `scripts/publish-vetrina.sh`**: dopo il push del branch
      deve pushare anche il **tag** `<label>` sul commit snapshot
      (`git tag -a "$LABEL" -m "Release $LABEL"` + `git push origin "$LABEL"`),
      perché `release.yml` scatta su push di tag. Prevedere idempotenza (tag già
      esistente ⇒ errore chiaro, non ri-push). Aggiornare `.vetrina/README.md`
      col nuovo flusso.
- [ ] **0.3 PR + merge su main** della modifica 0.2.

## Fase 1 — Vetrina pubblica e prima release (nome ancora `PromptAPorter`)

> Prima si rende pubblica, POI si builda: così la build 3 OS è gratis.

- [ ] **1.1 Snapshot v0.8.35 dal tag esatto** (non da HEAD, che è più avanti):
      ```bash
      git switch --detach v0.8.35
      VETRINA_REPO=https://github.com/robertomarchioro/PromptAPorter.git \
        ./scripts/publish-vetrina.sh v0.8.35   # con la 0.2 pusha anche il tag
      git switch main
      ```
      Nota: in detached HEAD lo script della 0.2 non c'è ancora (il tag v0.8.35
      precede la modifica) — usare la copia aggiornata da main:
      `git show main:scripts/publish-vetrina.sh > /tmp/pv.sh && bash /tmp/pv.sh …`
      oppure pushare il tag a mano dopo lo snapshot.
- [ ] **1.2 Rendere pubblica la vetrina** (il gate segreti dello script è già
      passato; eventuale doppio check manuale sul contenuto pushato):
      ```bash
      gh repo edit robertomarchioro/PromptAPorter --visibility public --accept-visibility-change-consequences
      ```
- [ ] **1.3 Replica gli 8 secret di release** sulla vetrina:
      ```bash
      for s in TAURI_SIGNING_PRIVATE_KEY TAURI_SIGNING_PRIVATE_KEY_PASSWORD \
               APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_ID \
               APPLE_PASSWORD APPLE_SIGNING_IDENTITY APPLE_TEAM_ID; do
        gh secret set "$s" -R robertomarchioro/PromptAPorter   # valore dal backup 0.1
      done
      ```
      `CERTUM_*` e `CLAUDE_CODE_OAUTH_TOKEN` NON vanno replicati
      (firma Windows locale; claude-code-review.yml escluso dalla vetrina).
- [ ] **1.4 Abilita GitHub Pages** sulla vetrina, source **GitHub Actions**
      (Settings → Pages, o `gh api -X POST repos/robertomarchioro/PromptAPorter/pages -f build_type=workflow`).
- [ ] **1.5 Release build**: il push del tag (1.1) fa partire `release.yml`
      sulla vetrina → draft con gli 8 asset (3 OS). Attendere verde.
- [ ] **1.6 Box firma Windows**: `git pull` del repo sul box, poi
      `sign-release.ps1 -Repo robertomarchioro/PromptAPorter` (il default punta
      ancora al nome vecchio: passare `-Repo` esplicito SOLO per questa release
      di transizione). Firma Authenticode + rigenerazione `latest.json`.
- [ ] **1.7 Promuovi la release a Latest** e verifica `latest.json`: 7 entry
      piattaforma, firma == `.sig`, URL asset su `PromptAPorter` (ok: dopo il
      rename resteranno serviti dal redirect GitHub; dalla release successiva
      verranno generati col nome nuovo).

## Fase 2 — Lo swap dei nomi (finestra di 404 di pochi minuti: innocua, l'updater ritenta)

> ORDINE OBBLIGATO: il nome va prima liberato dal dev, poi assunto dalla
> vetrina. Il redirect del nome vecchio decade quando il nome viene riusato —
> è esattamente l'effetto voluto.

- [ ] **2.1** `gh repo rename prompt-a-porter-dev -R robertomarchioro/prompt-a-porter`
- [ ] **2.2** `gh repo edit robertomarchioro/prompt-a-porter-dev --visibility private --accept-visibility-change-consequences`
      ⚠️ Irreversibile nei dettagli: star e watcher vanno persi, il sito Pages
      del dev si spegne, eventuali fork pubblici vengono scollegati.
      ⚠️ **Pages**: dal 2026-07-11 il sito con la landing è servito proprio dal
      dev → da qui resta GIÙ finché 3.1 non lo rideploya dalla vetrina.
      Prerequisito: 1.4 già fatto; eseguire 3.1 subito dopo lo swap.
- [ ] **2.3** `gh repo rename prompt-a-porter -R robertomarchioro/PromptAPorter`
- [ ] **2.4 SUBITO: aggiorna i remote locali su TUTTE le macchine** (box dev
      Linux, box firma Windows se ha un clone). Un push col vecchio URL
      finirebbe sul repo PUBBLICO:
      ```bash
      git remote set-url origin git@github.com:robertomarchioro/prompt-a-porter-dev.git
      ```
- [ ] **2.5 Verifiche post-swap**:
      ```bash
      curl -sIL https://github.com/robertomarchioro/prompt-a-porter/releases/latest/download/latest.json | head -1   # 200
      gh repo view robertomarchioro/prompt-a-porter --json visibility     # PUBLIC
      gh repo view robertomarchioro/prompt-a-porter-dev --json visibility # PRIVATE
      ```
      E dal vivo: un'installazione v0.8.34 deve vedere e installare la v0.8.35.

## Fase 3 — Assestamento

- [ ] **3.1 Sito**: lancia `site-deploy.yml` sulla vetrina (workflow_dispatch)
      e verifica `https://robertomarchioro.github.io/prompt-a-porter/`
      (il `base: "/prompt-a-porter/"` di VitePress torna corretto col nome nuovo).
- [ ] **3.2 Issues**: abilita le issue sulla vetrina (i template in
      `.github/ISSUE_TEMPLATE/` arrivano con lo snapshot). Decidere dove vive il
      lavoro interno (issue sul dev privato restano com'erano).
- [ ] **3.3 CI sul dev privato** (ora consuma il monte 2.000 min/mese, tutto
      Linux ×1): disabilita `site-deploy.yml`; valuta se tenere gli schedule
      `dep-canary.yml` e `security-audit.yml` sul dev o spostarli mentalmente
      sulla vetrina (ci girano già gratis a ogni snapshot).
- [ ] **3.4 `sign-release.ps1`**: il default `-Repo 'robertomarchioro/prompt-a-porter'`
      ora punta correttamente al repo pubblico — nessuna modifica necessaria,
      da 1.6 in poi non serve più passare `-Repo`.
- [ ] **3.5 Nuovo processo release** (da v0.8.36 in poi):
      1. bump + CHANGELOG + commit su dev → tag `vX.Y.Z` su dev (storico interno);
      2. `VETRINA_REPO=git@github.com:robertomarchioro/prompt-a-porter.git ./scripts/publish-vetrina.sh vX.Y.Z`
         → snapshot + tag → `release.yml` builda sulla vetrina;
      3. firma sul box Windows + promozione a Latest (invariato).
      ⚠️ `release.yml` sul **dev** scatterebbe anche lì sul tag: disattivarlo o
      limitarlo sul dev (build 3 OS su privato = minuti fatturati, macOS ×10).
- [ ] **3.6 Doc e memoria**: aggiorna questo file a migrazione completata,
      più `docs/contribuire/release-signing-workflow.md` se descrive il flusso.

## Punti di attenzione trasversali

- **Release storiche** (≤ v0.8.35 sul dev): con il dev privato i vecchi link di
  download nel CHANGELOG muoiono per il pubblico. Accettato; la storia pubblica
  riparte dalla v0.8.35 sulla vetrina.
- **Ordine 1.x prima di 2.x**: mai fare lo swap dei nomi senza una release
  Latest già pronta sulla vetrina, o gli utenti esistenti restano senza
  updater funzionante più a lungo del necessario.
- **Gate segreti**: lo scan di `publish-vetrina.sh` resta l'unico cancello tra
  la storia interna e il pubblico. Non aggirarlo mai con push manuali sulla
  vetrina.
