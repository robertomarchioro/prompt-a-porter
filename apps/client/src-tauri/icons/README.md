# Icone Prompt a Porter

Glifo: **Lucide `braces`** (ISC license, compatibile GPL 2.0).
Icona app: braces bianche su sfondo ambra con gradiente.

## Sorgenti SVG

- `app-icon.svg` — Icona app 1024×1024 (sfondo ambra + braces bianche)
- `tray-icon.svg` — Icona tray 32×32 (monocromatica, sfondo trasparente)

## Generare i PNG

### Metodo 1: Tool HTML (consigliato)

1. Apri `genera-icone.html` nel browser
2. Clicca "Scarica" su ogni dimensione per salvare i PNG
3. Salva `icon.png` (1024×1024) in questa directory

### Metodo 2: Tauri CLI

Dopo aver ottenuto `icon.png` (1024×1024):

```bash
pnpm tauri icon icons/icon.png
```

Genera automaticamente: `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`.

## File necessari

- `icon.png` — 1024×1024 sorgente
- `32x32.png` — tray icon / favicon
- `128x128.png` — icona app
- `128x128@2x.png` — icona app retina
- `icon.icns` — macOS
- `icon.ico` — Windows
