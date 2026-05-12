# Roadmap thread website

> Milestone del thread parallelo "Landing page PaP". Cinque fasi sequenziali, indipendenti dalla roadmap del prodotto (v1.0/v2.0).

## Stato globale

🟡 **Thread aperto, in attesa di assegnazione agente parallelo.** Tutta la documentazione è pronta (vedi [README.md](./README.md)).

## Fasi

### Fase 1 — Setup tecnico

Obiettivo: struttura Astro inizializzata e build pipeline GitHub Pages funzionante (anche con contenuto placeholder).

- [ ] Crea sottocartella `website/` nel repo come da [`tech-stack.md`](./tech-stack.md) §"Struttura"
- [ ] `pnpm create astro@latest` + integrazioni (Tailwind, TypeScript)
- [ ] `astro.config.mjs` configurato per GitHub Pages
- [ ] Workflow `.github/workflows/website-deploy.yml` come da [`hosting.md`](./hosting.md) §"Step 2"
- [ ] Abilitato GitHub Pages → Settings → Pages → Source: GitHub Actions
- [ ] First deploy verde con pagina "Coming soon" placeholder
- [ ] URL live: `https://robertomarchioro.github.io/prompt-a-porter/`

**Tempo stimato**: 0.5-1 giorno.
**Bloccanti**: nessuno.

### Fase 2 — Handoff a Claude Design

Obiettivo: ricevere mockup high-fidelity per le 8 sezioni della landing.

- [ ] Passa [`contenuti.md`](./contenuti.md) integralmente a Claude Design
- [ ] Richiesta esplicita di:
  - Mockup desktop 1440px + mobile 375px per 8 sezioni
  - Palette tonale (zinc/slate/stone) + dark/light variants
  - Tipografia (sizes h1-h6, body, mono)
  - Componenti riutilizzabili (card, CTA, hero, comparison table)
  - Screenshot prodotto preparati
  - Logo + favicon + OG image
- [ ] Iterazione 1-2 round con feedback
- [ ] Approvazione finale dei mockup

**Tempo stimato**: 1-2 settimane (dipende dalla disponibilità di Claude Design).
**Bloccanti**: nessuno tecnico.

### Fase 3 — Sviluppo

Obiettivo: implementare la landing seguendo i mockup, con copy IT da [`contenuti.md`](./contenuti.md).

- [ ] Importa palette + tipografia in `tailwind.config.mjs`
- [ ] Implementa `<Base.astro>` layout con meta tag SEO, favicon, OG
- [ ] Implementa componenti riutilizzabili: `Hero`, `CapabilityCard`, `PersonaCard`, `ComparisonTable`, `CTAButton`
- [ ] Implementa `pages/index.astro` aggregando le 8 sezioni
- [ ] Copy IT integrale da [`contenuti.md`](./contenuti.md)
- [ ] Screenshot del prodotto in `public/screenshots/` (ottimizzati webp)
- [ ] Mobile responsive verificato breakpoint 375/768/1024/1440px
- [ ] Dark mode (se Claude Design la prevede)
- [ ] Snippet `APP_VERSION` letta da `apps/client/package.json` a build-time
- [ ] Link "Scarica" punta a `https://github.com/robertomarchioro/prompt-a-porter/releases/latest`
- [ ] Lighthouse score verificato ≥ 95 in Performance/Accessibility/SEO

**Tempo stimato**: 3-5 giorni.
**Bloccanti**: Fase 2 chiusa.

### Fase 4 — Setup analytics

Obiettivo: Matomo container live su Gigantto + integrazione frontend funzionante.

- [ ] Deploy container `docker-compose.matomo.yml` come da [`analytics-matomo.md`](./analytics-matomo.md)
- [ ] Setup wizard Matomo completato (super user, primo sito, ID)
- [ ] Configurazione privacy: IP anonymization, DNT support, Force SSL
- [ ] Tracking snippet in `Base.astro` con `MATOMO_SITE_ID` corretto
- [ ] Goal configurati (4 voci tabella in `analytics-matomo.md` §"Goal")
- [ ] Verifica live: page view + eventi CTA registrati su staging
- [ ] Footer disclaimer Matomo + link `/privacy` opzionale
- [ ] Backup cron weekly DB schedulato

**Tempo stimato**: 0.5-1 giorno (setup container + integrazione).
**Bloccanti**: Fase 3 in produzione (per testare tracking end-to-end).

### Fase 5 — Lancio + manutenzione continua

Obiettivo: landing live su URL definitivo, manutenzione async con prodotto.

- [ ] Decisione su custom domain (registra subito o resta su `github.io` subdomain inizialmente)
- [ ] Se custom domain: setup DNS come da [`hosting.md`](./hosting.md) §"Setup DNS"
- [ ] Annuncio lancio su canali (Twitter/Mastodon/Hacker News/Reddit r/LocalLLaMA)
- [ ] Aggiungi badge "🌐 Website" al README del repo principale
- [ ] Aggiungi link landing nella tabella README repo
- [ ] Monitoraggio prime 2 settimane: bounce rate, top sezioni, CTA conversion

**Manutenzione ricorrente** (post-lancio):
- Ad ogni release prodotto significativa (es. v1.0): aggiornamento copy "Le capability" + screenshot
- Ogni mese: review numeri Matomo, decisioni iterative (es. spostare sezione, riscrivere copy)
- Quando v2.0 Enterprise diventa concreto: refresh totale (probabile nuovo branch/template)

## Dipendenze tra fasi

```
Fase 1 (setup) ──┐
                 ├──> Fase 3 (sviluppo) ──> Fase 4 (analytics) ──> Fase 5 (lancio)
Fase 2 (design) ─┘
```

Fase 1 e Fase 2 possono partire in parallelo. Fase 3 attende entrambe.

## Quando far partire il thread

**Non è urgente** rispetto al prodotto. Suggerimenti:

- **Subito**: se vuoi una pagina "Coming soon v1.0" già su `*.github.io` per cominciare a indicizzare con Google e raccogliere referrer.
- **Parallelo a M2-M3 di v1.0**: timing naturale, la landing diventa pronta più o meno quando v1.0 GA.
- **Solo dopo v1.0 GA**: rischio di lanciare prodotto senza website. Sconsigliato.

**Raccomandazione**: parallelo a M2-M3. Avvia Fase 1+2 quando i layer UI di v1.0 (a11y, sub-step Fase 4) sono in corso. Quando v1.0.0 GA, la landing è già live.

## Effort totale stimato

- Fase 1: 0.5-1g
- Fase 2: ~2 settimane (waiting time)
- Fase 3: 3-5g
- Fase 4: 0.5-1g
- Fase 5: 1g + manutenzione ricorrente

**Lavoro effettivo dell'agente parallelo**: ~6-10 giorni nell'arco di 3-4 settimane (con downtime di waiting su Claude Design).

## Manutenzione di questo documento

- Quando una fase chiude: spuntare le voci della checklist.
- Quando emerge nuova attività (es. blog post di lancio, video demo): aggiungere come Fase 6.
- Quando il thread va in idle (es. landing stabile, nessun cambio prodotto): segnalare in §"Stato globale" con data.
