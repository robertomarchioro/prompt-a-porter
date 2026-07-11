# Hosting landing page

> **Hosting scelto**: GitHub Pages.
> **Costo**: zero. Unlimited bandwidth e build per repository pubblici.
> **Dominio**: `robertomarchioro.github.io/prompt-a-porter` inizialmente. Custom domain opzionale dopo (~€10/anno).

## GitHub Pages — perché

| Criterio | GitHub Pages | Alternative |
|---|---|---|
| **Costo** | $0 (unlimited bandwidth per repo pubblici) | Cloudflare Pages $0, Netlify $0, Vercel $0 — tutti free tier sufficienti |
| **CI/Deploy** | GitHub Actions integrato, workflow ufficiale | Stesso livello altrove |
| **Co-locazione codice** | ✅ stesso repo del prodotto, niente sync repo separati | Altri richiedono o repo separato o Git import |
| **Custom domain** | ✅ supportato con HTTPS auto via Let's Encrypt | Idem |
| **CDN globale** | ✅ Fastly | Cloudflare/Netlify hanno edge proprio |
| **Lock-in** | Minimo: il build statico è portable, deploy altrove in 5 minuti | Idem |

Per una landing open-source ospitata nel repo stesso del prodotto, GitHub Pages è la scelta naturale: niente account extra, niente integrazioni terze, niente sync di stato.

## Configurazione iniziale

### Step 1 — Abilita GitHub Pages

Su `github.com/robertomarchioro/prompt-a-porter` → Settings → Pages:
- **Source**: GitHub Actions (NON "Deploy from a branch", che è il vecchio metodo)
- Lascia il resto invariato.

### Step 2 — Workflow GitHub Actions

Crea `.github/workflows/website-deploy.yml`:

```yaml
name: Deploy landing page

on:
  push:
    branches: [main]
    paths:
      - 'website/**'
      - '.github/workflows/website-deploy.yml'
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: website
    steps:
      - uses: actions/checkout@v4

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'
          cache-dependency-path: website/pnpm-lock.yaml

      - name: Install deps
        run: pnpm install --frozen-lockfile

      - name: Build Astro
        run: pnpm build

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: website/dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

**Note importanti**:
- `paths:` filter assicura che il workflow scatti solo per modifiche a `website/`, no rebuild gratuiti su ogni commit di prodotto.
- `concurrency` impedisce 2 deploy paralleli che si pestano i piedi.
- `permissions` minimi necessari per GitHub Pages OIDC token.

### Step 3 — Verifica deploy

Dopo il primo push su `main` toccando `website/`, la landing è live su:
- `https://robertomarchioro.github.io/prompt-a-porter/`

Build time atteso: 30-60 secondi.

## Dominio

### Fase iniziale: subdomain GitHub Pages

URL: `robertomarchioro.github.io/prompt-a-porter`

Pro: zero costo, zero setup DNS.
Contro: estetica meno pulita, path subdir invece di root.

`astro.config.mjs` deve avere:
```javascript
base: '/prompt-a-porter',
```

### Fase successiva: custom domain

Quando si decide di registrare un dominio (~€10/anno):

**Provider economici** (in ordine indicativo):
- **Cloudflare Registrar** — vende dominio a prezzo wholesale, no markup. Ottimo.
- **Namecheap** — buon equilibrio prezzo/qualità.
- **Porkbun** — economico, no upsell.
- Evitare GoDaddy (markup alto, upsell aggressivo).

**Nomi candidati** (da verificare disponibilità):
- `promptaporter.dev` (estensione `.dev` adatta per progetti tech)
- `promptaporter.app`
- `pap-prompts.io`
- `prompt-a-porter.it`

### Setup DNS per custom domain

Su GitHub: Settings → Pages → Custom domain → inserisci `promptaporter.dev` (esempio).

Su provider DNS:

**Record per apex domain** (`promptaporter.dev`):
```
A     @  185.199.108.153
A     @  185.199.109.153
A     @  185.199.110.153
A     @  185.199.111.153
```

**Record per www** (opzionale):
```
CNAME www robertomarchioro.github.io.
```

GitHub Pages emette automaticamente cert TLS Let's Encrypt entro pochi minuti.

`astro.config.mjs` con custom domain diventa:
```javascript
site: 'https://promptaporter.dev',
base: '/',  // o rimuovi
```

## Opzioni alternative archiviate

Per riferimento futuro, in caso si voglia migrare:

### Cloudflare Pages
- **Free tier**: 500 build/mese, unlimited bandwidth, edge global.
- **Pro**: performance superiore (edge), analytics web built-in gratis, preview deploy per ogni PR.
- **Quando preferirlo a GitHub Pages**: se in futuro la landing diventa molto trafficata o se si decide di usare Cloudflare Web Analytics invece di Matomo.

### Netlify
- **Free tier**: 100 GB bandwidth/mese, 300 build min/mese.
- **Pro**: Netlify Forms gratis (utile se aggiungiamo form contatto in futuro), preview deploy.
- **Quando preferirlo**: se serve form submission senza backend.

### Vercel
- **Free tier**: 100 GB bandwidth/mese, optimized per Next.js (meno per Astro statico).
- **Quando preferirlo**: se il sito evolve verso SSR/ISR (improbabile per landing).

## Considerazioni operative

### Quanto carico aspettarsi

Per una landing open-source di un progetto nuovo:
- Primi 3 mesi: ~100-500 visite/mese (Hacker News, Reddit, post Twitter/Mastodon).
- Anno 1: ~1k-5k visite/mese se il progetto guadagna traction.

GitHub Pages regge ordini di grandezza superiori senza problemi.

### Backup

Niente: il sito è statico, ricostruibile da `website/` repo. Se GitHub Pages va giù, il sorgente è in `main` e si può deployare ovunque in 10 minuti.

### Costi futuri (se si volesse)

- **Custom domain**: ~€10/anno (Cloudflare Registrar o Namecheap)
- **Email su dominio custom** (es. `roberto@promptaporter.dev`): Cloudflare Email Routing gratis, oppure Fastmail ~€3/mese, oppure self-host (ma complica)
- **CDN aggiuntivo**: non necessario (GitHub Pages CDN già adeguato)
- **Premium analytics**: non necessario (Matomo self-host copre)

**Totale costo annuo**: €0-10 (solo dominio se lo si vuole).

## Manutenzione

- Quando GitHub Actions upgradano una versione: aggiorna `actions/checkout@v4` → `@v5` etc. quando esce.
- Quando Astro rilascia major version: testa upgrade in branch, poi merge.
- Verifica trimestrale che il workflow di deploy giri pulito (no warning Node version deprecated, etc.).
