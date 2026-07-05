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
#
# NOTA STORICA (2 regressioni "gate always-pass" già capitate su questo
# script, occhio a non reintrodurle):
#   1) il pattern inizia con "-----BEGIN..." che grep scambierebbe per
#      un'opzione da riga di comando (leading "-") senza il flag `-e`
#      esplicito — grep falliva con "unrecognized option", l'errore finiva
#      su /dev/null e `|| true` lo inghiottiva, HITS restava sempre vuoto.
#      Fixato con `-e "$SECRET_PATTERNS"`.
#   2) $ALLOWLIST_FILE contiene righe vuote/commenti. Con `grep -F` un
#      pattern-file con una riga VUOTA fa sì che il pattern vuoto matchi
#      OGNI riga in input, e con `-v` questo esclude TUTTO l'output (anche
#      segreti reali). Fixato pre-pulendo l'allowlist (via
#      allowlist_clean_file, sotto) esattamente come si fa già per
#      $EXCLUDE_FILE, e saltando `grep -vF` del tutto quando la lista
#      ripulita è vuota (nessun pattern -> nessun filtro, non un match-all).
#      Vedi scripts/test-publish-vetrina-gate.sh per il test di regressione.
SECRET_PATTERNS='-----BEGIN (RSA|OPENSSH|EC|DSA|PGP|ENCRYPTED)? ?PRIVATE KEY-----'
SECRET_PATTERNS="$SECRET_PATTERNS"'|ghp_[A-Za-z0-9]{36}|gho_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{60,}'
SECRET_PATTERNS="$SECRET_PATTERNS"'|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{12,}|AIza[0-9A-Za-z_\-]{35}'
# Anthropic / OpenAI / Stripe / SendGrid
SECRET_PATTERNS="$SECRET_PATTERNS"'|sk-ant-[A-Za-z0-9_\-]{20,}|sk-proj-[A-Za-z0-9_\-]{20,}|sk-[A-Za-z0-9]{20,}'
SECRET_PATTERNS="$SECRET_PATTERNS"'|sk_live_[A-Za-z0-9]{20,}|rk_live_[A-Za-z0-9]{20,}|SG\.[A-Za-z0-9_.\-]{20,}'
# Assegnazioni generiche password/secret/api-key con valore non banale
SECRET_PATTERNS="$SECRET_PATTERNS"'|([Pp]assword|[Ss]ecret|[Aa]pi[_-]?[Kk]ey)[[:space:]]*[:=][[:space:]]*["'"'"'][^"'"'"']{8,}["'"'"']'

# Allowlist ripulita (niente righe vuote/commenti) — vedi NOTA STORICA (2)
# sopra. Se non resta nessun pattern valido, ALLOWLIST_CLEAN_FILE è vuoto:
# in quel caso NON invocare `grep -vF` (pattern-file vuoto = comportamento
# non garantito/pericoloso), semplicemente non filtrare nulla (`cat`).
ALLOWLIST_CLEAN_FILE="$STAGING/allowlist.clean.txt"
grep -vE '^[[:space:]]*(#|$)' "$ALLOWLIST_FILE" > "$ALLOWLIST_CLEAN_FILE" || true
filter_allowlist() {
  if [ -s "$ALLOWLIST_CLEAN_FILE" ]; then
    grep -vFf "$ALLOWLIST_CLEAN_FILE" || true
  else
    cat
  fi
}

# `-a`/--binary-files=text (al posto di `-I`): un segreto in un file con
# anche un solo byte non-UTF8 (o un formato binario come .pfx/.p12/DER)
# verrebbe classificato "binary" da grep e SALTATO con `-I`. Forziamo la
# scansione testuale di tutto.
# `|| true` sull'INTERA pipe (non solo dentro filter_allowlist): con
# `pipefail` attivo, se il grep principale non trova nulla (exit 1, il
# caso "nessun segreto" — quello buono) la pipe restituirebbe comunque
# stato non-zero anche se filter_allowlist ha successo, facendo uscire lo
# script sotto `set -e` PRIMA di stampare "Gate segreti: OK".
HITS="$(grep -rnE -a -e "$SECRET_PATTERNS" "$BUILD" 2>/dev/null | filter_allowlist || true)"

# --- 2b. Euristica blob base64 ad alta entropia (es. chiave privata Ed25519
# Tauri Updater). Scansione ristretta ai file tracciati NON lockfile: i
# lockfile (Cargo.lock, go.sum, pnpm-lock.yaml, ecc.) contengono hash
# esadecimali/sha512-base64 lunghi che darebbero falsi positivi sistematici.
# Scartiamo anche i match puramente esadecimali (probabili checksum/hash,
# non segreti in formato base64 "vero" che include lettere g-z/G-Z, '+',
# '/' o '=' di padding).
BASE64_MIN_LEN=60
LOCK_FILE_REGEX='(^|/)(Cargo\.lock|go\.sum|pnpm-lock\.yaml|package-lock\.json|yarn\.lock)$'
BASE64_HITS=""
while IFS= read -r f; do
  if printf '%s' "$f" | grep -qE "$LOCK_FILE_REGEX"; then
    continue
  fi
  [ -f "$BUILD/$f" ] || continue
  while IFS= read -r tok; do
    case "$tok" in
      *[!0-9a-fA-F]*) ;; # contiene char non-esadecimale -> possibile segreto
      *) continue ;;      # solo hex -> probabile checksum/hash, skip
    esac
    BASE64_HITS="${BASE64_HITS}${f}: blob base64 sospetto (${#tok} char, prefisso ${tok:0:12}...)"$'\n'
  done < <(grep -a -oE "[A-Za-z0-9+/]{${BASE64_MIN_LEN},}=?=?" "$BUILD/$f" 2>/dev/null || true)
done < "$STAGING/files.txt"
if [ -n "$BASE64_HITS" ]; then
  BASE64_HITS="$(printf '%s' "$BASE64_HITS" | filter_allowlist || true)"
fi

if [ -n "$HITS" ] || [ -n "$BASE64_HITS" ]; then
  printf '\033[31m✗ GATE SEGRETI: possibili segreti reali — pubblicazione ABORTITA.\033[0m\n' >&2
  [ -n "$HITS" ] && printf '%s\n' "$HITS" >&2
  [ -n "$BASE64_HITS" ] && printf '%s' "$BASE64_HITS" >&2
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
