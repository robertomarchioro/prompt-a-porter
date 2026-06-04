#!/usr/bin/env bash
#
# publish-vetrina.sh — pubblica uno snapshot sanitizzato del repo sul repo
# "vetrina" pubblico. PROIEZIONE PURA: copia (git ls-files − esclusioni) →
# gate scan segreti → commit snapshot → push. Nessuna riscrittura di contenuti
# (la sorgente è già "publish-clean").
#
# Storia del repo pubblico: un commit "Release <label>" per ogni pubblicazione
# (snapshot per release), niente storia di sviluppo interna.
#
# USO:
#   VETRINA_REPO=git@github.com:owner/repo.git ./scripts/publish-vetrina.sh v1.0.0
#   DRY_RUN=1 ./scripts/publish-vetrina.sh v1.0.0    # build + gate, nessun push
#
# Se <label> è omesso, usa il tag esatto su HEAD (git describe --exact-match).
#
# CONFIG (env):
#   VETRINA_REPO     URL git del repo vetrina (obbligatorio se DRY_RUN!=1)
#   VETRINA_BRANCH   branch di destinazione (default: main)
#   DRY_RUN          1 = non clona/pusha, mostra solo cosa farebbe
#   EXCLUDE_FILE     default .vetrina/exclude.txt
#   ALLOWLIST_FILE   default .vetrina/secret-allowlist.txt
#
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

VETRINA_REPO="${VETRINA_REPO:-}"
VETRINA_BRANCH="${VETRINA_BRANCH:-main}"
DRY_RUN="${DRY_RUN:-0}"
EXCLUDE_FILE="${EXCLUDE_FILE:-.vetrina/exclude.txt}"
ALLOWLIST_FILE="${ALLOWLIST_FILE:-.vetrina/secret-allowlist.txt}"

err() { printf '\033[31mERRORE:\033[0m %s\n' "$*" >&2; exit 1; }
info() { printf '\033[36m›\033[0m %s\n' "$*"; }

# --- Label release ---
LABEL="${1:-$(git describe --tags --exact-match 2>/dev/null || true)}"
[ -n "$LABEL" ] || err "Nessun tag esatto su HEAD e nessuna label passata come argomento."

# --- Validazioni ---
[ -f "$EXCLUDE_FILE" ] || err "Manifest esclusioni non trovato: $EXCLUDE_FILE"
[ -f "$ALLOWLIST_FILE" ] || err "Allowlist non trovata: $ALLOWLIST_FILE"
# Solo modifiche TRACCIATE bloccano: lo snapshot usa `git ls-files` (file
# tracciati a HEAD), quindi file non-tracciati vaganti non lo alterano.
[ -z "$(git status --porcelain --untracked-files=no)" ] || err "Modifiche tracciate non committate: committa o stasha prima di pubblicare."
if [ "$DRY_RUN" != "1" ] && [ -z "$VETRINA_REPO" ]; then
  err "VETRINA_REPO non impostato (richiesto se DRY_RUN!=1)."
fi

STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT
BUILD="$STAGING/tree"
mkdir -p "$BUILD"

# --- 1. Lista file inclusi (tracked − esclusioni) ---
mapfile -t EXCLUDES < <(grep -vE '^[[:space:]]*(#|$)' "$EXCLUDE_FILE")
git ls-files | while IFS= read -r f; do
  skip=0
  for ex in "${EXCLUDES[@]}"; do
    # match esatto (file) oppure prefisso (directory con / finale).
    # if espliciti, non `A && B`: sotto `set -e` un && che valuta falso
    # come ultimo comando del corpo farebbe uscire la subshell del pipe.
    case "$f" in
      "$ex") skip=1; break ;;
      "$ex"*)
        if [ "${ex%/}" != "$ex" ]; then skip=1; break; fi
        ;;
    esac
  done
  if [ "$skip" = 0 ]; then printf '%s\n' "$f"; fi
done > "$STAGING/files.txt"

N="$(wc -l < "$STAGING/files.txt" | tr -d ' ')"
[ "$N" -gt 0 ] || err "Nessun file selezionato — controlla il manifest esclusioni."
info "File inclusi: $N (su $(git ls-files | wc -l | tr -d ' ') tracciati)"

rsync -a --files-from="$STAGING/files.txt" ./ "$BUILD"/

# --- 2. Gate scan segreti (pattern ad alta confidenza, con allowlist) ---
# Solo pattern di segreti REALI per evitare falsi-abort; i casi noti innocui
# stanno in $ALLOWLIST_FILE. Estendi i pattern se servono altri formati.
SECRET_PATTERNS='-----BEGIN (RSA|OPENSSH|EC|DSA|PGP|ENCRYPTED)? ?PRIVATE KEY-----|ghp_[A-Za-z0-9]{36}|gho_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{60,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{12,}|AIza[0-9A-Za-z_\-]{35}'
HITS="$(grep -rInE "$SECRET_PATTERNS" "$BUILD" 2>/dev/null | grep -vFf "$ALLOWLIST_FILE" || true)"
if [ -n "$HITS" ]; then
  printf '\033[31m✗ GATE SEGRETI: possibili segreti reali — pubblicazione ABORTITA.\033[0m\n' >&2
  printf '%s\n' "$HITS" >&2
  printf '\nSe sono falsi positivi, aggiungili a %s dopo verifica.\n' "$ALLOWLIST_FILE" >&2
  exit 2
fi
info "Gate segreti: OK (nessun segreto reale)."

# --- 3. Dry-run: stop qui ---
if [ "$DRY_RUN" = "1" ]; then
  info "[DRY-RUN] Snapshot '$LABEL' pronto. Nessun clone/push."
  info "Top-level che verrebbe pubblicato:"
  ( cd "$BUILD" && ls -A )
  exit 0
fi

# --- 4. Clone vetrina, sync snapshot, commit, push ---
CLONE="$STAGING/vetrina"
info "Clono $VETRINA_REPO ($VETRINA_BRANCH)…"
if ! git clone --quiet --branch "$VETRINA_BRANCH" "$VETRINA_REPO" "$CLONE" 2>/dev/null; then
  # branch non esistente (repo vuoto / primo push)
  git clone --quiet "$VETRINA_REPO" "$CLONE"
  ( cd "$CLONE" && git checkout -q -B "$VETRINA_BRANCH" )
fi

# Rimuovi il contenuto tracciato precedente (mantieni .git), poi inserisci lo snapshot
( cd "$CLONE" && git rm -rq --ignore-unmatch . ) >/dev/null 2>&1 || true
rsync -a --delete --exclude='.git/' "$BUILD"/ "$CLONE"/

cd "$CLONE"
git add -A
if git diff --cached --quiet; then
  info "Nessuna differenza rispetto alla vetrina attuale — niente da pubblicare."
  exit 0
fi
git commit -q -m "Release $LABEL"
git push --quiet origin "HEAD:$VETRINA_BRANCH"
info "✓ Pubblicato snapshot '$LABEL' su $VETRINA_REPO ($VETRINA_BRANCH)."
