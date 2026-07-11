#!/usr/bin/env bash
#
# setup-ubuntu.sh — Setup guidato workstation Ubuntu 24.04 per
# sviluppo Prompt a Porter + gestione chiavi Tauri Updater.
#
# Componenti gestiti (idempotente, salta cio' che e' gia' installato):
#   - apt: build-essential, curl, git, file, libssl-dev, pkg-config,
#          libwebkit2gtk-4.1-dev, libayatana-appindicator3-dev,
#          librsvg2-dev, libxdo-dev, patchelf  (tutto cio' che serve a
#          buildare Tauri 2 su Linux)
#   - GitHub CLI (gh) via repo ufficiale cli.github.com
#   - Node.js LTS via NodeSource setup script
#   - pnpm via corepack (incluso in Node.js >=16)
#   - Tauri CLI (@tauri-apps/cli) globale via pnpm
#
# NOTA: Authenticode signing (signtool.exe) NON e' disponibile su
# Linux. Questa workstation puo':
#   - Sviluppare e buildare l'app
#   - Generare/custodire la chiave privata Tauri Updater
#   - Eseguire `tauri signer sign` per re-signing Ed25519
# Non puo' eseguire scripts/sign-release.ps1 (richiede Windows +
# SimplySign Desktop GUI). Per la firma Authenticode usa
# scripts/setup-windows.ps1 + scripts/sign-release.ps1.
#
# Usage:
#   ./scripts/setup-ubuntu.sh                # interattivo
#   ./scripts/setup-ubuntu.sh --skip-install # solo verifica + env
#   ./scripts/setup-ubuntu.sh --no-tauri-deps # skip libs Tauri Linux

set -euo pipefail

SKIP_INSTALL=false
NO_TAURI_DEPS=false
for arg in "$@"; do
  case "$arg" in
    --skip-install) SKIP_INSTALL=true ;;
    --no-tauri-deps) NO_TAURI_DEPS=true ;;
    -h|--help)
      sed -n '2,30p' "$0"
      exit 0
      ;;
    *) echo "Argomento sconosciuto: $arg"; exit 1 ;;
  esac
done

section() {
  echo ""
  echo "==========================================================="
  echo "  $1"
  echo "==========================================================="
}

cmd_exists() {
  command -v "$1" >/dev/null 2>&1
}

# 1. Preflight
section "1. Preflight"
if [[ "$(id -u)" -eq 0 ]]; then
  echo "[FAIL] non eseguire come root. Lo script usa sudo solo dove serve."
  exit 1
fi
if ! cmd_exists lsb_release; then
  sudo apt-get update && sudo apt-get install -y lsb-release
fi
DISTRO_ID="$(lsb_release -is)"
DISTRO_VER="$(lsb_release -rs)"
echo "Distribuzione: $DISTRO_ID $DISTRO_VER"
if [[ "$DISTRO_ID" != "Ubuntu" ]]; then
  echo "[WARN] script testato su Ubuntu. Procedo comunque ma alcuni passi potrebbero fallire."
fi

# 2. apt update + base packages
if ! $SKIP_INSTALL; then
  section "2. apt: pacchetti base"
  sudo apt-get update
  sudo apt-get install -y \
    build-essential \
    curl \
    git \
    file \
    ca-certificates \
    gnupg
  echo "[OK] pacchetti base"

  # 2b. Dipendenze Tauri 2 Linux (libwebkit2gtk-4.1 ecc.)
  if ! $NO_TAURI_DEPS; then
    section "2b. apt: dipendenze Tauri 2 Linux"
    sudo apt-get install -y \
      libssl-dev \
      pkg-config \
      libwebkit2gtk-4.1-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev \
      libxdo-dev \
      patchelf
    echo "[OK] dipendenze Tauri Linux"
  else
    echo "[INFO] --no-tauri-deps: skip libs Tauri"
  fi

  # 3. GitHub CLI da repo ufficiale (cfr. https://github.com/cli/cli/blob/trunk/docs/install_linux.md)
  if ! cmd_exists gh; then
    section "3. GitHub CLI (gh)"
    sudo mkdir -p -m 755 /etc/apt/keyrings
    out=$(mktemp)
    wget -nv -O"$out" https://cli.github.com/packages/githubcli-archive-keyring.gpg
    cat "$out" | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null
    sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg
    rm "$out"
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
      | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
    sudo apt-get update
    sudo apt-get install -y gh
    echo "[OK] gh installato"
  else
    echo "[OK] gh gia' installato: $(gh --version | head -1)"
  fi

  # 4. Node.js LTS via NodeSource
  if ! cmd_exists node; then
    section "4. Node.js LTS"
    # NOTA SICUREZZA: non usiamo piu' la URL "rolling"
    # https://deb.nodesource.com/setup_lts.x pipeata direttamente in
    # `sudo bash` (il contenuto puo' cambiare in qualsiasi momento senza
    # preavviso, senza modo di verificarne l'integrita'). Pinniamo invece
    # a un commit specifico e immutabile del repo nodesource/distributions
    # e verifichiamo lo sha256 dello script scaricato prima di eseguirlo.
    # Per aggiornare il pin (es. nuova LTS major): scarica manualmente
    # https://deb.nodesource.com/setup_lts.x, ispezionalo, poi aggiorna
    # NODESOURCE_SETUP_URL (nuovo commit SHA) e NODESOURCE_SETUP_SHA256
    # qui sotto.
    NODESOURCE_SETUP_URL="https://raw.githubusercontent.com/nodesource/distributions/c6e581b0d24e5d043476ddb947d70e6fe10e83c9/scripts/deb/setup_lts.x"
    NODESOURCE_SETUP_SHA256="6e3d580f5bd7ccf2aa1e8df8d35c60d78e873c3ff8beb282c9bebd914904ad72"
    nodesource_script="$(mktemp)"
    curl -fsSL "$NODESOURCE_SETUP_URL" -o "$nodesource_script"
    if ! echo "$NODESOURCE_SETUP_SHA256  $nodesource_script" | sha256sum -c - >/dev/null 2>&1; then
      echo "[FAIL] checksum NodeSource setup script non corrisponde al pin atteso."
      echo "       Non eseguo lo script. Vedi commento sopra per aggiornare il pin."
      rm -f "$nodesource_script"
      exit 1
    fi
    sudo -E bash "$nodesource_script"
    rm -f "$nodesource_script"
    sudo apt-get install -y nodejs
    echo "[OK] node: $(node -v)"
  else
    echo "[OK] node gia' installato: $(node -v)"
  fi

  # 5. pnpm via corepack
  section "5. pnpm via corepack"
  sudo corepack enable
  corepack prepare pnpm@latest --activate
  echo "[OK] pnpm: $(pnpm -v)"

  # 6. Tauri CLI globale
  section "6. Tauri CLI (@tauri-apps/cli)"
  if cmd_exists tauri; then
    echo "[OK] tauri gia' presente: $(tauri -V)"
  else
    pnpm i -g @tauri-apps/cli
    # pnpm global bin potrebbe non essere in PATH al primo install
    if ! cmd_exists tauri; then
      PNPM_BIN="$(pnpm bin -g)"
      echo "[WARN] 'tauri' non in PATH. pnpm global bin: $PNPM_BIN"
      echo "       Aggiungi al tuo ~/.bashrc:"
      echo "         export PATH=\"\$PATH:$PNPM_BIN\""
      echo "       Poi: source ~/.bashrc"
    else
      echo "[OK] tauri: $(tauri -V)"
    fi
  fi
fi

# 7. Verifica gh CLI auth
section "7. Verifica gh CLI auth"
if cmd_exists gh; then
  if gh auth status >/dev/null 2>&1; then
    echo "[OK] gh CLI autenticato"
  else
    echo "[INFO] gh CLI non autenticato. Esegui: gh auth login"
    read -r -p "Eseguire 'gh auth login' adesso? [Y/n] " ans
    if [[ ! "$ans" =~ ^[nN] ]]; then
      gh auth login
    fi
  fi
fi

# 8. Chiave Tauri Updater
section "8. Chiave privata Tauri Updater"
TAURI_DIR="$HOME/.tauri"
KEY_PATH="$TAURI_DIR/pap-updater.key"
PUB_PATH="$KEY_PATH.pub"

mkdir -p "$TAURI_DIR"
chmod 700 "$TAURI_DIR"

if [[ -f "$KEY_PATH" ]]; then
  echo "[OK] chiave presente: $KEY_PATH"
  if [[ -f "$PUB_PATH" ]]; then
    echo "    pubkey: $(cat "$PUB_PATH")"
  fi
else
  echo "Chiave NON presente in $KEY_PATH"
  echo ""
  echo "OPZIONI:"
  echo "  A) Hai gia' la chiave (es. backup, password manager, altra macchina):"
  echo "     copia il file in $KEY_PATH e rilancia ./scripts/setup-ubuntu.sh --skip-install"
  echo ""
  echo "  B) Sei al primo setup del progetto e devi GENERARE una chiave nuova:"
  echo "     ATTENZIONE - la chiave Tauri Updater e' la chiave 'eterna' del progetto."
  echo "     Se ne esiste gia' una in uso (vedi tauri.conf.json -> plugins.updater.pubkey)"
  echo "     NON generarne una nuova: gli utenti con l'app installata non potrebbero"
  echo "     piu' ricevere update finche' non installano manualmente la nuova versione."
  echo ""
  read -r -p "Generare nuova chiave Ed25519? [y/N] " ans
  if [[ "$ans" =~ ^[yY] ]]; then
    if ! cmd_exists tauri; then
      echo "[FAIL] tauri CLI non in PATH. Riapri shell e rilancia."
      exit 1
    fi
    read -r -p "Password per cifrare la chiave (vuoto = no password): " -s KEY_PWD
    echo ""
    # NOTA (#466): la passphrase NON viene passata a `tauri` come
    # argomento -p (finirebbe in chiaro in /proc/*/cmdline, leggibile da
    # altri processi dello stesso utente). A differenza di `signer sign`,
    # `signer generate` NON legge la password dalla env var
    # TAURI_SIGNING_PRIVATE_KEY_PASSWORD (verificato: se settata senza -p
    # genera silenziosamente una chiave NON protetta) — quindi qui non
    # possiamo fare la stessa scorciatoia di sign-release.ps1. L'unica
    # via sicura e' il prompt interattivo nativo della CLI (letto da
    # /dev/tty, mai da argv/env): se e' stata scelta una password sopra,
    # va reinserita quando richiesto qui sotto.
    if [[ -n "$KEY_PWD" ]]; then
      echo "[INFO] tauri chiedera' ora la password: reinseriscila (la stessa appena digitata sopra)."
      tauri signer generate -w "$KEY_PATH"
    else
      tauri signer generate -w "$KEY_PATH" -p ""
    fi
    echo ""
    echo "[OK] chiave generata in $KEY_PATH"
    echo "[OK] pubkey in $PUB_PATH:"
    cat "$PUB_PATH"
    echo ""
    if [[ -n "$KEY_PWD" ]]; then
      if cmd_exists secret-tool; then
        # Passphrase custodita nel keyring di sistema (GNOME Keyring /
        # KWallet via libsecret), non in chiaro su disco. Vedi sezione 9
        # per come viene riletta nella shell.
        secret-tool store --label="Prompt a Porter - Tauri Updater key passphrase" \
          service prompt-a-porter account tauri-updater-key <<<"$KEY_PWD"
        echo "[OK] passphrase salvata nel keyring (secret-tool)"
      else
        echo "[WARN] secret-tool (libsecret-tools) non trovato: passphrase NON salvata."
        echo "       Installa con: sudo apt-get install libsecret-tools"
        echo "       poi rilancia, oppure salvala manualmente in un password manager."
      fi
    fi
    unset KEY_PWD
    echo "PROSSIMI STEP:"
    echo "  1. Backup di $KEY_PATH in password manager (Bitwarden, 1Password, etc.)"
    echo "  2. Aggiorna apps/client/src-tauri/tauri.conf.json:"
    echo "     plugins.updater.pubkey = '<contenuto di $PUB_PATH single-line>'"
    echo "  3. Aggiorna GitHub Secrets:"
    echo "     TAURI_SIGNING_PRIVATE_KEY = (contenuto di $KEY_PATH)"
    echo "     TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (password sopra, se settata)"
  fi
fi

# 9. Aggiorna .bashrc con env vars utili (idempotente)
section "9. Env vars in ~/.bashrc"
BASHRC="$HOME/.bashrc"
MARKER="# >>> prompt-a-porter setup >>>"
END_MARKER="# <<< prompt-a-porter setup <<<"

if grep -q "$MARKER" "$BASHRC" 2>/dev/null; then
  echo "[OK] blocco gia' presente in $BASHRC (skip)"
else
  cat >> "$BASHRC" <<EOF

$MARKER
# Path alla chiave privata Tauri Updater (usata da scripts/sign-release.ps1
# su Windows; su Linux serve per tauri signer sign / generate).
export TAURI_UPDATER_PRIVATE_KEY_PATH="\$HOME/.tauri/pap-updater.key"
# Se la chiave Updater e' cifrata, la passphrase NON va scritta qui in
# chiaro. Viene invece custodita nel keyring di sistema (GNOME Keyring /
# KWallet via libsecret) e riletta a ogni apertura di shell:
#   salvarla:   secret-tool store --label="Prompt a Porter - Tauri Updater key passphrase" \\
#                 service prompt-a-porter account tauri-updater-key
#   rimuoverla: secret-tool clear service prompt-a-porter account tauri-updater-key
if command -v secret-tool >/dev/null 2>&1; then
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD="\$(secret-tool lookup service prompt-a-porter account tauri-updater-key 2>/dev/null)"
  export TAURI_SIGNING_PRIVATE_KEY_PASSWORD
fi
$END_MARKER
EOF
  echo "[OK] aggiunto blocco in $BASHRC"
  echo "    Per attivarlo nella shell corrente: source ~/.bashrc"
fi

# 10. Riepilogo
section "10. Riepilogo"
echo ""
echo "Componenti:"
printf "  gh CLI:      %s\n" "$(cmd_exists gh && gh --version | head -1 || echo MISSING)"
printf "  Node.js:     %s\n" "$(cmd_exists node && node -v || echo MISSING)"
printf "  pnpm:        %s\n" "$(cmd_exists pnpm && pnpm -v || echo MISSING)"
printf "  Tauri CLI:   %s\n" "$(cmd_exists tauri && tauri -V || echo MISSING)"
printf "  signtool:    n/a (Windows-only)\n"
echo ""
echo "Chiave Tauri Updater:"
if [[ -f "$KEY_PATH" ]]; then
  printf "  %s (presente)\n" "$KEY_PATH"
else
  printf "  (non presente in %s)\n" "$KEY_PATH"
fi
echo ""
echo "Prossimi step:"
echo "  1. source ~/.bashrc  (o riapri terminale)"
echo "  2. Per buildare l'app:    cd <repo> && pnpm install && pnpm tauri dev"
echo "  3. Per firmare release:   serve workstation Windows + scripts/sign-release.ps1"
echo ""
echo "Riferimenti:"
echo "  docs/contribuire/setup-sviluppo.md           (setup dev completo)"
echo "  docs/contribuire/setup-tauri-updater-keys.md (chiavi Updater)"
echo "  docs/contribuire/release-signing-workflow.md (signing release)"
