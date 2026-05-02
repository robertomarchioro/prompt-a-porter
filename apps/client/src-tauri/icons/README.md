# Icone Prompt a Porter

Questa directory deve contenere le icone dell'app nei formati richiesti da Tauri.

## File necessari

- `icon.png` — 1024×1024 sorgente
- `32x32.png` — tray icon / favicon
- `128x128.png` — icona app
- `128x128@2x.png` — icona app retina
- `icon.icns` — macOS
- `icon.ico` — Windows

## Generazione

Partendo da un `icon.png` sorgente 1024×1024:

```bash
pnpm tauri icon icon.png
```

Questo comando genera automaticamente tutti i formati necessari.
