# Analytics — Matomo self-hosted

> **Soluzione**: Matomo Community Edition come container Docker su **Gigantto** (infrastruttura container personale dell'utente).
> **Costo**: zero (Gigantto è già up).
> **Privacy**: full data ownership, GDPR-friendly out-of-the-box, no Google.

## Matomo — perché

| Criterio | Matomo self-host | Google Analytics 4 | Plausible cloud | Cloudflare Web Analytics |
|---|---|---|---|---|
| **Costo** | $0 (Gigantto già up) | $0 (ma costo "shadow": dati ceduti a Google) | $9/mese | $0 (lock-in CF) |
| **Privacy** | ✅ full ownership | ❌ dati a Google | ✅ anonimo | ✅ no cookie |
| **GDPR** | ✅ out-of-the-box (IP anonymization opzionale) | richiede cookie banner complesso | ✅ | ✅ |
| **Funzionalità** | Complete: heatmap, session replay, A/B test, goal | Estese ma intrusive | Minimali (page view + referrer) | Minimali |
| **Cookie banner** | Opzionale (no se anonymized) | Obbligatorio | No | No |
| **Lock-in** | Zero (i dati sono nostri) | Massimo | Basso | Medio |
| **Open source** | ✅ GPL | ❌ | ✅ AGPL | ❌ |

Matomo è la scelta naturale quando si ha già infrastruttura container: massima sovranità sui dati, copertura funzionale enterprise, allineamento perfetto con il "privacy first" che la landing comunica.

## Architettura

```
┌──────────────────────────────────┐
│ Browser visitatore               │
│ promptaporter.* / *.github.io    │
└────────────┬─────────────────────┘
             │ matomo.js (~16 KB gz)
             │ HTTPS POST /matomo.php
             ▼
┌──────────────────────────────────┐
│ Gigantto                         │
│ ┌──────────────────────────────┐ │
│ │ Reverse proxy (Caddy/Traefik)│ │
│ │ → matomo.tld → matomo:80     │ │
│ └────────────┬─────────────────┘ │
│              │                   │
│ ┌────────────▼─────────────────┐ │
│ │ matomo container (php-fpm +   │ │
│ │ nginx, image matomo:latest)   │ │
│ └────────────┬─────────────────┘ │
│              │ MySQL protocol    │
│ ┌────────────▼─────────────────┐ │
│ │ mariadb container             │ │
│ │ (volume persistente)          │ │
│ └──────────────────────────────┘ │
└──────────────────────────────────┘
```

## Stack container (Docker Compose)

File `docker-compose.matomo.yml` su Gigantto:

```yaml
version: '3.8'

services:
  matomo-db:
    image: mariadb:11
    container_name: matomo-db
    restart: unless-stopped
    environment:
      MARIADB_ROOT_PASSWORD: ${MATOMO_DB_ROOT_PASSWORD}
      MARIADB_DATABASE: matomo
      MARIADB_USER: matomo
      MARIADB_PASSWORD: ${MATOMO_DB_PASSWORD}
    volumes:
      - matomo-db-data:/var/lib/mysql
    networks:
      - matomo-net

  matomo:
    image: matomo:latest
    container_name: matomo-app
    restart: unless-stopped
    depends_on:
      - matomo-db
    environment:
      MATOMO_DATABASE_HOST: matomo-db
      MATOMO_DATABASE_USERNAME: matomo
      MATOMO_DATABASE_PASSWORD: ${MATOMO_DB_PASSWORD}
      MATOMO_DATABASE_DBNAME: matomo
    volumes:
      - matomo-app-data:/var/www/html
    networks:
      - matomo-net
      - reverse-proxy   # rete condivisa col reverse proxy esistente
    labels:
      # esempio per Traefik (adatta se usi Caddy/nginx)
      - "traefik.enable=true"
      - "traefik.http.routers.matomo.rule=Host(`matomo.tuo-dominio.it`)"
      - "traefik.http.routers.matomo.entrypoints=websecure"
      - "traefik.http.routers.matomo.tls.certresolver=letsencrypt"
      - "traefik.http.services.matomo.loadbalancer.server.port=80"

volumes:
  matomo-db-data:
  matomo-app-data:

networks:
  matomo-net:
  reverse-proxy:
    external: true
```

Variabili in `.env`:

```bash
MATOMO_DB_ROOT_PASSWORD=...lungo random...
MATOMO_DB_PASSWORD=...lungo random...
```

Risorse stimate:
- **RAM**: ~512 MB Matomo + ~256 MB MariaDB = ~768 MB totali
- **CPU**: low idle, picchi su reporting
- **Storage**: ~500 MB initial + crescita lenta (~10 MB/mese per landing low-traffic)

## Setup iniziale

1. **Deploy container**:
   ```bash
   cd /opt/gigantto/matomo
   cp .env.example .env
   nano .env  # imposta password robuste
   docker compose -f docker-compose.matomo.yml up -d
   ```

2. **Setup wizard** Matomo: aprire `https://matomo.tuo-dominio.it`, primo accesso lancia il wizard:
   - Step 1: System Check (verde se tutto OK)
   - Step 2: Database setup (auto-pickup da env)
   - Step 3: Super User creation (email + password admin)
   - Step 4: Add a website → `promptaporter.dev` o `robertomarchioro.github.io/prompt-a-porter`
   - Step 5: JavaScript Tracking Code → copiare per la landing

3. **Configurazione privacy** (Settings → Privacy):
   - ☑ Anonymize visitor IPs (default ON, 2 bytes mascherati)
   - ☑ Anonymize "Referer" tracking
   - ☑ Enable Do Not Track support
   - ☑ Force SSL for tracking
   - ☐ Delete old visitor logs (lascia OFF: i dati sono nostri, non scadono)

4. **Disabilita features non necessarie** (Plugins): rimuovere CustomVariables, Provider (geo IP lookup), Insights se non utili → meno carico.

## Integrazione nella landing Astro

Inserire il tracking code in `src/layouts/Base.astro`, dentro `<head>`:

```astro
---
const MATOMO_URL = 'https://matomo.tuo-dominio.it/';
const MATOMO_SITE_ID = '1';  // sostituire con l'ID dello wizard step 4
const isProd = import.meta.env.PROD;
---
<head>
  <!-- ...altri meta tag... -->

  {isProd && (
    <script is:inline define:vars={{ MATOMO_URL, MATOMO_SITE_ID }}>
      var _paq = (window._paq = window._paq || []);
      _paq.push(['trackPageView']);
      _paq.push(['enableLinkTracking']);
      (function () {
        var u = MATOMO_URL;
        _paq.push(['setTrackerUrl', u + 'matomo.php']);
        _paq.push(['setSiteId', MATOMO_SITE_ID]);
        var d = document,
          g = d.createElement('script'),
          s = d.getElementsByTagName('script')[0];
        g.async = true;
        g.src = u + 'matomo.js';
        s.parentNode.insertBefore(g, s);
      })();
    </script>
  )}
</head>
```

**Note**:
- `is:inline` impedisce ad Astro di bundlerizzare lo script (necessario per Matomo, che si auto-inietta).
- `define:vars` passa le costanti di build come variabili runtime.
- `isProd` check evita tracking in dev locale (`pnpm dev`).

## Goal e conversion tracking

Configurare in Matomo → Goals i seguenti goal:

| Goal | Trigger | Valore |
|---|---|---|
| **Download Windows portable** | URL contiene `releases/latest` o `Prompt-a-Porter-portable` | 1 |
| **Click GitHub repo** | URL outlink contiene `github.com/robertomarchioro/prompt-a-porter` | 1 |
| **Click CHANGELOG** | URL outlink contiene `CHANGELOG.md` | 0.5 |
| **Scroll oltre 50%** | Event "scroll-depth-50" (richiede plugin Matomo Heatmap) | 0.3 |

Eventi custom in JS (esempio CTA Hero):

```astro
<a href={downloadUrl} class="btn-primary" onclick="_paq.push(['trackEvent', 'CTA', 'Download', 'Hero', 1])">
  Scarica per Windows
</a>
```

## Cosa tracciare (e cosa NO)

**SÌ — Page view, eventi UI critici, goal**:
- Pageview di ogni pagina (default)
- Click CTA Hero (Download / GitHub)
- Click outlink (GitHub, CHANGELOG, release page)
- Scroll depth (anonimo, aggregato)
- Referrer (anonymized)

**NO — niente di personale**:
- Niente form submission (non abbiamo form in v1)
- Niente identificatori utente
- Niente fingerprinting
- Niente integrazione social tracking
- Niente cookie obbligatori (se IP anonymized: cookie law non si applica)

## Cookie banner: serve o no?

**No**, se configurato correttamente:
- IP anonymized (2 bytes mascherati) → no PII raccolto
- Cookie `_pk_*` sono "tecnici" sotto la deroga del GDPR Art. 6.1.f (legitimate interest) per analytics first-party anonymized
- Do Not Track rispettato → utenti consapevoli sono autoescludenti

**Disclaimer** breve nel footer è comunque buona pratica:
> Questa pagina usa Matomo self-hosted per analytics anonimizzati. Nessun cookie di terze parti. Nessun dato condiviso. [Maggiori info](/privacy).

Una pagina `/privacy` dedicata (5 paragrafi max) chiude il loop legale.

## Manutenzione

- **Aggiornamento Matomo**: ogni 1-2 mesi esce una minor. `docker compose pull && docker compose up -d` su Gigantto. Backup DB prima.
- **Backup MariaDB**: cron weekly `mysqldump matomo > /backups/matomo-$(date +%F).sql.gz`. Conservare ultimi 4 backup.
- **Verifica integrità**: ogni mese controllare in Matomo che il tracking continui a registrare dati (no zero events sospetti).
- **Rotate password DB**: ogni 6 mesi (best practice security).

## Failure modes

| Scenario | Conseguenza | Mitigazione |
|---|---|---|
| Gigantto offline | Landing funziona, tracking perso per durata downtime | Accettabile: la landing è statica e non dipende da Matomo |
| Container Matomo crash | Tracking perso fino a restart | `restart: unless-stopped` riavvia auto. Alert via monitoring esistente Gigantto |
| MariaDB corruption | Dati storici persi | Backup weekly mitiga al peggio 7 giorni di dati |
| Subdomain `matomo.tuo-dominio.it` non risolvibile | Tracking fallisce silenzioso | Browser ignora errore, niente UX impact |

In tutti i casi la landing continua a funzionare — Matomo è additivo, non bloccante.

## Alternative se Matomo si rivela troppo pesante

Se in futuro Gigantto si satura o si decide di liberare risorse:

- **GoatCounter self-host**: container Go, ~50 MB RAM, schema flat-file. Funzionalità ridotte ma sufficienti per landing.
- **Umami self-host**: container Node + PostgreSQL, ~256 MB. Bello visivamente, meno feature di Matomo.
- **Plausible self-host**: container Elixir + PostgreSQL, ~400 MB. Stesso footprint di Matomo ma UI più moderna.
- **Cloudflare Web Analytics**: zero overhead Gigantto (solo se host landing su Cloudflare Pages).

Migrazione storica dei dati: Matomo esporta CSV, ogni alternativa ha import path documentato.

## Manutenzione di questo documento

- Quando setup Matomo è completato e live: spuntare gli step + aggiornare URL effettivo `matomo.tuo-dominio.it`.
- Quando si aggiungono goal nuovi: aggiungere alla tabella §"Goal".
- Quando si cambia approccio (es. migrazione altro tool): aggiornare §"Alternative".
