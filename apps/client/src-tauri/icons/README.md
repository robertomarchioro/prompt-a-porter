# Icone Prompt-à-porter

Segno: **`{ P }`** — graffe del prompt (JetBrains Mono, opacità 72%) che
vestono la P couture (Newsreader serif), su squircle viola con gradiente
`#8470D5 → #6E56CF`. Asset definitivi dal handoff design in
`docs/roadmap/icons/` (palette `violet/`, la primaria).

## Provenienza dei file

- `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.icns`,
  `icon.ico` — copiati **così come sono** da
  `docs/roadmap/icons/violet/tauri/` (rasterizzazioni finali con
  centratura ottica; non rigenerarli).
- `64x64.png`, `Square*Logo.png`, `StoreLogo.png`, `android/`, `ios/` —
  generati con `pnpm tauri icon docs/roadmap/icons/violet/icon-1024.png`
  dalla stessa sorgente 1024.

## Rigenerare (solo i derivati)

```bash
cd apps/client
pnpm tauri icon ../../docs/roadmap/icons/violet/icon-1024.png
# poi ricopiare sopra i 6 file finali da docs/roadmap/icons/violet/tauri/
```

## Note

- La tray icon usa a runtime l'icona di default della finestra
  (compilata da `bundle.icon` in tauri.conf.json): nessun asset
  dedicato.
- Per i dettagli di design (token, costruzione del segno, palette ambra
  alternativa) vedi `docs/roadmap/icons/README.md`.
