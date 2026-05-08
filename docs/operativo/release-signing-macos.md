# Release signing & notarization macOS

> **Status**: bloccato — Apple Developer enrollment in corso (KYC).
> Procedura da eseguire **dopo** che l'enroll è attivo.

Questa runbook copre l'intero percorso operativo per arrivare ad asset
macOS firmati `Developer ID Application` e notarizzati Apple, in modo che
gli utenti possano lanciare `Prompt.app` con doppio-click standard senza
il workaround "tasto destro → Apri".

Lo stato attuale del progetto:

- Build cross-OS già attivi in `.github/workflows/release.yml` (Linux,
  macOS aarch64, Windows).
- macOS aarch64 produce `Prompt.a.Porter_<ver>_aarch64.dmg` +
  `Prompt.a.Porter_aarch64.app.tar.gz` **non firmati e non notarizzati**.
- `release.yml` riga 85 contiene la nota workaround per gli utenti.
- Nessun secret macOS configurato in GitHub.

## Prerequisiti (sblocco esterno)

- ✅ Iscrizione Apple Developer Program completata (~$99/anno).
- ⏳ Enrollment attivo (24-48h se Individual, fino a 1 settimana
  Organization). Email "Welcome" da Apple.
- 🖥 Accesso a un Mac (anche prestato, una tantum) per generare il
  certificato — non si può fare da Linux/Windows.

Quando l'enroll è attivo, segui questa procedura **una sola volta**.

## 1. Sull'Apple Developer Portal

1. Verifica enrollment attivo: developer.apple.com → l'account deve
   risultare attivo.
2. **Trova il Team ID** (10 caratteri, es. `ABC1234DEF`):
   Apple Developer → Membership Details. Lo userai come secret
   `APPLE_TEAM_ID`.

## 2. Genera il certificato `Developer ID Application`

Su un Mac:

3. Apri **Keychain Access** → menu Keychain Access → Certificate
   Assistant → **Request a Certificate from a Certificate Authority**.
   Salva il `.certSigningRequest` (CSR) sul desktop.
4. Apple Developer → **Certificates, IDs & Profiles → Certificates → +**.
   Scegli **`Developer ID Application`** (NON "Developer ID Installer",
   NON "Mac App Store"). Carica il CSR. Scarica il `.cer`.
5. Doppio-click sul `.cer` per importarlo in Keychain. Apparirà sotto
   "My Certificates" con la sua chiave privata.
6. **Esporta come `.p12`**: tasto destro sul cert → Export → formato
   Personal Information Exchange (.p12). Imposta una password robusta
   (es. da password manager). Conserva il file.
7. **Converti `.p12` in base64** per GitHub:

   ```sh
   # macOS
   base64 -i DeveloperID.p12 | pbcopy

   # Linux
   base64 DeveloperID.p12 -w0
   ```

## 3. Credenziali per `notarytool` — API Key (raccomandato)

L'alternativa "App-Specific Password" è ancora supportata ma deprecata.
Usa una API Key per `notarytool`.

8. **App Store Connect → Users and Access → Integrations → Keys → +**
   (sezione **Team Keys**). Ruolo richiesto: **Developer**.
9. Scarica il `.p8` (download disponibile **una sola volta**, conservalo).
10. Annota **Key ID** (10 char) e **Issuer ID** (UUID).

## 4. Configura GitHub Actions secrets

Vai su `Settings → Secrets and variables → Actions` del repo. Aggiungi:

| Secret | Valore |
|---|---|
| `APPLE_CERTIFICATE` | output del base64 (passo 7) |
| `APPLE_CERTIFICATE_PASSWORD` | password del `.p12` |
| `APPLE_SIGNING_IDENTITY` | nome esatto del cert, es. `Developer ID Application: <Tuo Nome> (ABC1234DEF)` |
| `APPLE_TEAM_ID` | Team ID (passo 2) |
| `APPLE_API_KEY` | contenuto del `.p8` (multi-line OK) |
| `APPLE_API_KEY_ID` | Key ID (passo 10) |
| `APPLE_API_ISSUER` | Issuer ID (passo 10) |

## 5. Modifica `release.yml`

`tauri-action` legge questi secret nativamente quando passati come `env`
al job macOS — niente custom signing logic da scrivere.

Modifica il job macOS in `.github/workflows/release.yml`:

```yaml
- name: build-tauri
  uses: tauri-apps/tauri-action@v0
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
    APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
    APPLE_API_KEY: ${{ secrets.APPLE_API_KEY }}
    APPLE_API_KEY_ID: ${{ secrets.APPLE_API_KEY_ID }}
    APPLE_API_ISSUER: ${{ secrets.APPLE_API_ISSUER }}
  with:
    # ...altri parametri esistenti
```

Tauri 2 nota la presenza di `APPLE_API_KEY` e attiva automaticamente
notarytool dopo il signing. Riferimento ufficiale:
https://v2.tauri.app/distribute/sign/macos/

## 6. Test della pipeline

11. **Tag patch dedicato** per testare prima della prossima minor:

    ```sh
    git tag -a v0.6.1 -m "test signing macOS"
    git push origin v0.6.1
    ```

    Triggera `release.yml` con tutti i 3 OS. Solo macOS userà i nuovi
    secret.

12. **Verifica artifact firmato** scaricando `.app` o `.dmg` dalla
    release draft:

    ```sh
    # Verifica firma codesign valida
    codesign --verify --deep --strict --verbose=2 /path/to/Prompt.app
    spctl -a -t exec -vv /path/to/Prompt.app

    # Verifica notarization stapled
    xcrun stapler validate /path/to/Prompt.app
    ```

    Output atteso: `accepted source=Notarized Developer ID`.

## 7. Cleanup post-success

13. **Rimuovi la nota workaround** in `release.yml` riga 85:

    ```diff
    - - **macOS**: alla prima apertura, tasto destro sull'app → "Apri"
    -   (necessario perché non notarizzato).
    + - **macOS**: trascina `.app` in `/Applications` e fai doppio-click
    +   per lanciare. App firmata Developer ID + notarizzata Apple.
    ```

14. **Crea ADR** `docs/architettura/decisioni/apple-notarization.md`:
    - Provider scelto (Apple Developer Program Individual)
    - Costo annuale e processo rinnovo
    - Cert expiry (Developer ID Application: 5 anni)
    - Secret rotation policy in caso di compromissione
    - Chi ha accesso ai secret GitHub

15. **Aggiorna `docs/roadmap/rinvii.md`** marcando atterrato l'item
    "macOS notarization" (referenziato in
    `docs/architettura/decisioni/onnx-bundle.md:12`).

## Effort stimato

| Fase | Tempo | Skill required |
|---|---|---|
| Step 1-2 (portal Apple) | 5 min | nessuno |
| Step 3-10 (cert + API key) | 30-45 min | accesso Mac |
| Step 11-14 (release.yml + ADR) | 1 PR (~30 min) | dev usuale |

Tutti gli ingredienti tecnici Tauri sono già pronti — la procedura è
gate amministrativo + 1 PR di config.

## Parallelo Windows: Authenticode Certum

Procedura analoga per il certificato Authenticode (firma `.exe`/`.msi`)
è bloccata sull'arrivo del cert Certum OSS (KYC in corso). Vedi
`docs/roadmap/rinvii.md` § "Patch line `v0.2.x`" e ADR pendente
`docs/architettura/decisioni/authenticode-signing.md` (da creare).

## Riferimenti esterni

- **Tauri 2 — macOS signing**:
  https://v2.tauri.app/distribute/sign/macos/
- **Apple — Notarizing macOS software before distribution**:
  https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
- **GitHub Actions — Encrypted secrets**:
  https://docs.github.com/en/actions/security-guides/encrypted-secrets

## Cronologia

- **2026-05-07** — Creato come runbook in attesa che Apple Developer
  enrollment si attivi.
