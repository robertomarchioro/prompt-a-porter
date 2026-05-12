# Tech stack landing page

> **Stack scelto**: Astro 4.x con output statico.
> **Razionale**: standard de-facto per landing open source nel 2026. Component-based, zero JS di default (image optimization, MDX, view transitions inclusi), build statico → deploy ovunque. Imparato con 1-2 ore.

## Astro — perché

| Criterio | Astro | Alternative considerate |
|---|---|---|
| **Zero JS di default** | ✅ Solo HTML+CSS in output salvo `client:*` directive esplicita | Next.js/SvelteKit: spediscono framework runtime |
| **Component-based** | ✅ Sintassi `.astro` simile a Svelte, riusa componenti tra sezioni | HTML puro: copia-incolla manuale |
| **Image optimization** | ✅ `<Image>` nativo, srcset+webp+avif automatici | 11ty/Hugo: plugin/short-code manuali |
| **MDX integrato** | ✅ Per blog/docs futuri | Hugo: shortcode, meno ergonomici |
| **Build statico** | ✅ Output `dist/` deployabile dovunque | SvelteKit: serverless complica deploy |
| **Curva di apprendimento** | Bassa (~1-2h se conosci HTML/JSX/Svelte) | Hugo: sintassi Go template più ostile |
| **Ecosistema** | Maturo (Tailwind, Astro Themes, integrations) | 11ty: maturo ma più "fai-da-te" |
| **Performance** | Lighthouse 100/100 out-of-the-box per landing | Tutti i moderni SSG ci arrivano |

## Versione e prerequisiti

- **Node.js** ≥ 20.10 (LTS).
- **pnpm** ≥ 9 (coerente con il client `apps/client`).
- **Astro** ultima stable della 4.x line.

## Struttura repository

L'agente parallelo decide tra 2 opzioni:

### Opzione A — Sottocartella nello stesso repo (preferita)

```
prompt-a-porter/
├── apps/
│   ├── client/
│   ├── server/
│   ├── mcp-server/
│   └── cli/
├── website/              ← nuovo
│   ├── astro.config.mjs
│   ├── package.json
│   ├── pnpm-lock.yaml
│   ├── public/
│   │   ├── favicon.svg
│   │   ├── apple-touch-icon.png
│   │   └── screenshots/
│   ├── src/
│   │   ├── components/
│   │   │   ├── Hero.astro
│   │   │   ├── CapabilityCard.astro
│   │   │   ├── PersonaCard.astro
│   │   │   ├── ComparisonTable.astro
│   │   │   └── ...
│   │   ├── layouts/
│   │   │   └── Base.astro
│   │   ├── pages/
│   │   │   └── index.astro
│   │   └── styles/
│   │       └── global.css
│   └── README.md
├── docs/
└── ...
```

Pro:
- Tutto in un repo, una review, una history.
- L'agente principale (sviluppo prodotto) vede subito quando l'agente parallelo aggiorna la landing.
- I link tra docs prodotto e landing sono semplici relative paths.

Contro:
- CI deve essere configurato con `paths:` filter perché solo modifiche a `website/` triggrino il deploy.
- Lock file separato (`pnpm-lock.yaml`) accanto a quello del client.

**Raccomandazione**: opzione A. È la scelta standard per progetti che vogliono mantenere coesione.

### Opzione B — Repo gemello

`prompt-a-porter-website` come repo separato.

Pro: completa indipendenza, ownership chiaro.
Contro: doppio repo da gestire, lo stato si desincronizza, link a docs/release più scomodi.

**Sconsigliata** salvo motivi specifici emersi durante lo sviluppo.

## Setup iniziale (agente parallelo, Fase 1)

```bash
cd prompt-a-porter
mkdir website && cd website

pnpm create astro@latest . --template minimal --typescript --git
# answer: install deps yes, init git no (siamo già in repo)

pnpm add -D @astrojs/tailwind tailwindcss
pnpm astro add tailwind

# i18n future-ready (anche se ora solo IT)
pnpm add -D astro-i18next
```

Editare `astro.config.mjs`:

```javascript
import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';

export default defineConfig({
  site: 'https://robertomarchioro.github.io',  // o custom domain
  base: '/prompt-a-porter',                     // se subdomain GitHub Pages
  integrations: [tailwind()],
  output: 'static',
  build: {
    inlineStylesheets: 'auto',
  },
});
```

## Component primer (esempi)

### `src/layouts/Base.astro`

```astro
---
const { title, description } = Astro.props;
---
<!doctype html>
<html lang="it">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{title}</title>
  <meta name="description" content={description} />
  <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
  <meta property="og:image" content="/og-image.png" />
  <!-- Matomo placeholder, vedi analytics-matomo.md -->
</head>
<body>
  <slot />
</body>
</html>
```

### `src/components/Hero.astro`

```astro
---
const downloadUrl = 'https://github.com/robertomarchioro/prompt-a-porter/releases/latest';
---
<section class="hero">
  <h1>Prompt a Porter</h1>
  <p class="lead">La tua libreria di prompt AI, sul tuo computer. Open source.</p>
  <div class="cta">
    <a href={downloadUrl} class="btn-primary">Scarica per Windows</a>
    <a href="https://github.com/robertomarchioro/prompt-a-porter" class="btn-secondary">Vedi su GitHub</a>
  </div>
</section>
```

## Build & dev

```bash
# Sviluppo
pnpm dev          # http://localhost:4321

# Build statico
pnpm build        # output in dist/

# Preview build
pnpm preview      # serve dist/ in locale
```

## Performance target

Lighthouse score atteso per la landing finale:

- **Performance**: ≥ 95
- **Accessibility**: ≥ 95
- **Best Practices**: 100
- **SEO**: 100

Astro out-of-the-box raggiunge quasi sempre questi numeri. Cose da tenere d'occhio:
- Immagini ottimizzate via `<Image>` Astro (no `<img>` raw).
- Font web caricati con `font-display: swap`.
- Niente JS lato client salvo strettamente necessario (esempio: animazione hero hover può essere CSS-only).
- Inline critical CSS via `build.inlineStylesheets: 'auto'`.

## Testing

Niente test unitari per la landing — è statica per natura. Quality gate:

- [ ] `pnpm build` verde
- [ ] `pnpm astro check` 0 errori
- [ ] Lighthouse CI in GitHub Actions verde sui target sopra
- [ ] Smoke test manuale su Chrome / Firefox / Safari
- [ ] Smoke test manuale su mobile (iOS Safari, Android Chrome)
- [ ] HTML validator W3C verde
- [ ] Link check (no 404 interni)

## Aggiornamenti automatici dei contenuti

Alcuni contenuti della landing dipendono dallo stato del prodotto:

- **URL "Scarica Windows"** → punta sempre a `releases/latest` di GitHub (statico).
- **Versione corrente** → letta da `apps/client/package.json` a build-time tramite Astro `import` o `fs.readFileSync` in `astro.config.mjs`.
- **Screenshot** → in `website/public/screenshots/`, aggiornati manualmente quando UI cambia significativamente.
- **CHANGELOG link** → punta a `CHANGELOG.md` su GitHub direttamente.

### Snippet per leggere versione da package.json a build-time

```javascript
// src/utils/version.ts
import clientPkg from '../../../apps/client/package.json' assert { type: 'json' };
export const APP_VERSION = clientPkg.version;
```

Poi nei componenti: `<p>Versione corrente: v{APP_VERSION}</p>`.

## Manutenzione

- Quando esce una release del prodotto: rebuild landing con il nuovo `APP_VERSION` (automatico via CI se trigger su tag).
- Quando UI del client cambia: aggiornare screenshot.
- Quando una capability nuova entra (es. v1.0 M4 import scopati): aggiungere card in [`contenuti.md`](./contenuti.md) §3.
