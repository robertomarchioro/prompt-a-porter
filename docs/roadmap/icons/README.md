# Handoff: Icona applicazione — Prompt‑à‑porter `{ P }`

## Overview
Icona **definitiva** di Prompt‑à‑porter (libreria di prompt AI "pronti da indossare"). Il segno **`{ P }`** fonde le due anime del prodotto: le **graffe** del prompt/codice (eredità dell'icona app) che *vestono* la **P couture** in serif (identità del sito). Il pacchetto contiene tutte le dimensioni e i formati necessari per web, macOS, Windows, iOS e Android, in due palette: **viola** (primaria) e **ambra** (alternativa calda).

## About the Design Files
Le **immagini** (`.png`, `.ico`, `.icns`) in questo bundle sono **asset finali, pronti alla produzione** — vanno copiate così come sono nel codebase, non ricreate. Le rasterizzazioni sono già state generate a partire dai font del brand (Newsreader + JetBrains Mono) con centratura ottica.

I file **`.html`** (`icon-set.html`, `icon-concepts.html`) sono **solo riferimenti visivi** (foglio di contatto e tavola dei concept) per capire scelte e contesto — non vanno spediti.

## Fidelity
**High‑fidelity.** Asset finali. Colori, geometria e centratura sono definitivi.

## Il segno — costruzione
- Composizione: `{` (JetBrains Mono, peso 400, opacità **72%**) · `P` (Newsreader serif, peso 400, opacità 100%) · `}` (JetBrains Mono, peso 400, opacità 72%).
- **Centratura ottica**: ogni glifo è centrato sul proprio bounding box reale (non sulla baseline), sia in orizzontale sia in verticale. Il gruppo `{ P }` è centrato nel quadrato.
- Forma icona: **squircle** (raggio ≈ 22,5% del lato) per le versioni con angoli (iOS/macOS/app). Le versioni **maskable** sono a fondo pieno quadrato con il segno rimpicciolito nella safe‑zone.
- Sfondo: gradiente lineare diagonale (≈150°). Sottile alone chiaro radiale in alto a sinistra per profondità.

## Design Tokens
**Palette viola (primaria)**
- Gradiente sfondo: `#8470D5` → `#6E56CF`
- Glifo: `#FFFFFF` (graffe a 72% di opacità)
- Theme color (manifest/PWA): `#8470D5`

**Palette ambra (alternativa)**
- Gradiente sfondo: `#E8C79A` → `#D8A463`
- Glifo: `#3A2B11` (marrone scuro; graffe a 72% di opacità)
- Theme color: `#D8A463`

**Comune**
- Background app/sito (dark): `#0B0B0C`
- Squircle radius: `22.5%` del lato
- Font glifi: `Newsreader` (P), `JetBrains Mono` (graffe)

## Contenuto del pacchetto
Struttura identica per `violet/` e `amber/`:

| File | Dimensioni | Uso |
|---|---|---|
| `icon-16 … icon-1024.png` | 16,32,48,64,128,180,192,256,512,1024 | PNG generici, squircle opaco |
| `apple-touch-icon.png` | 180×180 | iOS Safari / “Aggiungi a Home” |
| `maskable-192.png`, `maskable-512.png` | 192, 512 | Android PWA (fondo pieno, safe‑zone) |
| `favicon.ico` | 16+32+48 (multi‑res) | Web / Windows |
| `icon.icns` | 16→1024 | App macOS |
| `site.webmanifest` | — | Manifest PWA già compilato |
| `tauri/` | vedi sotto | Nomi standard per Tauri/desktop |

`tauri/` contiene: `32x32.png`, `128x128.png`, `128x128@2x.png` (=256), `icon.png` (=512), `icon.ico`, `icon.icns`.

## Come usarle

### Sito web (favicon + PWA)
Usa la palette **viola**. Nel `<head>`:
```html
<link rel="icon" href="/icons/favicon.ico" sizes="any">
<link rel="icon" type="image/png" sizes="32x32" href="/icons/icon-32.png">
<link rel="icon" type="image/png" sizes="16x16" href="/icons/icon-16.png">
<link rel="apple-touch-icon" sizes="180x180" href="/icons/apple-touch-icon.png">
<link rel="manifest" href="/icons/site.webmanifest">
<meta name="theme-color" content="#8470D5">
```
Copia il contenuto di `violet/` in `public/icons/` (i percorsi nel `site.webmanifest` sono relativi, quindi vanno tenuti nella stessa cartella).

### App desktop Tauri (`apps/client`)
Sostituisci il contenuto di `apps/client/src-tauri/icons/` con i file di `violet/tauri/`. In `tauri.conf.json` la sezione `bundle.icon` deve elencare:
```json
"icon": ["icons/32x32.png","icons/128x128.png","icons/128x128@2x.png","icons/icon.icns","icons/icon.ico"]
```
> Nota: l'attuale `apps/client/src-tauri/icons/` contiene solo `128x128@2x.png` (vecchia icona `{}` ambra). Va rimpiazzato con il set completo qui fornito.

### macOS (app nativa non‑Tauri)
Usa `violet/icon.icns` come `AppIcon` (o importa i PNG in un `.iconset` / Assets catalog).

### iOS / Android
- iOS: `apple-touch-icon.png` (web) o i PNG nell'Asset Catalog (native).
- Android PWA: i `maskable-*.png` sono già dichiarati con `"purpose": "maskable"` nel `site.webmanifest`.

## Quando usare quale palette
- **Viola** = default ovunque (coerente con sito e brand primario).
- **Ambra** = solo se serve una variante calda/stagionale; stesso segno, stessa geometria.

## Nota sul favicon 16px
A 16px `{ P }` resta leggibile ma denso. Se in futuro si vuole un favicon più pulito ai minimi, si può usare la sola **P** come mark ridotto (pratica standard icona piena vs. mark). Non incluso in questo set — generabile su richiesta.

## Files (riferimenti visivi, NON da spedire)
- `icon-set.html` — foglio di contatto: tutte le taglie/formati su fondi macOS/iOS/chiaro.
- `icon-concepts.html` — tavola dei 6 concept iniziali con motivazione della scelta.

## Rigenerazione
Gli asset sono rasterizzazioni. Per rigenerarli servono i font **Newsreader** e **JetBrains Mono** (Google Fonts), il glifo `{ P }` con graffe a 72% e centratura sul bounding box reale, squircle a 22,5%, e i gradienti sopra. Un vettore SVG con testo convertito in tracciati può essere prodotto su richiesta se serve una sorgente scalabile.
