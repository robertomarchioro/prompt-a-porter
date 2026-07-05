#!/usr/bin/env bash
#
# test-publish-vetrina-gate.sh — test di regressione per il gate segreti di
# scripts/publish-vetrina.sh.
#
# Il gate e' "fallito aperto" (sempre pass, anche con segreti reali) DUE
# volte in passato:
#   1) grep scambiava il pattern "-----BEGIN..." per un'opzione CLI
#      (leading "-") senza `-e` esplicito -> "unrecognized option" ->
#      stderr su /dev/null -> `|| true` inghiottiva tutto.
#   2) l'allowlist conteneva righe vuote; con `grep -vF` un pattern vuoto
#      matcha OGNI riga in input, e con `-v` questo esclude TUTTO
#      l'output (anche segreti reali).
#
# Questo script costruisce repo git temporanei isolati (mai il repo reale)
# e verifica che il gate:
#   (a) fallisca (exit != 0) su un repo con un segreto reale tracciato
#       (AWS key, chiave privata OpenSSH, segreto in file "binario")
#   (b) passi (exit 0) su un repo pulito
#
# Uso:
#   ./scripts/test-publish-vetrina-gate.sh
#
# Exit 0 se tutti gli assert passano, exit 1 altrimenti (dettagli su stdout).
# Va rilanciato ad ogni modifica del gate segreti in publish-vetrina.sh.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_SCRIPT="$SCRIPT_DIR/publish-vetrina.sh"

PASS=0
FAIL=0

log_pass() { printf '\033[32m✓ PASS\033[0m %s\n' "$*"; PASS=$((PASS + 1)); }
log_fail() { printf '\033[31m✗ FAIL\033[0m %s\n' "$*"; FAIL=$((FAIL + 1)); }

# Crea un repo git temporaneo minimo con i file richiesti da
# publish-vetrina.sh (.vetrina/exclude.txt + .vetrina/secret-allowlist.txt).
# L'allowlist include DI PROPOSITO una riga vuota + commenti, per
# riprodurre esattamente le condizioni della regressione (2) sopra: il
# gate deve restare efficace comunque.
make_test_repo() {
  local dir
  dir="$(mktemp -d)"
  git -C "$dir" init -q
  git -C "$dir" config user.email test@example.com
  git -C "$dir" config user.name "Test"
  mkdir -p "$dir/.vetrina"
  cat >"$dir/.vetrina/exclude.txt" <<'EOF'
# nessuna esclusione nel test
EOF
  cat >"$dir/.vetrina/secret-allowlist.txt" <<'EOF'
# Allowlist di test — riga vuota intenzionale sotto per riprodurre la
# regressione (2): il gate deve restare efficace comunque.

# fixture innocua, non deve matchare nulla nei test
totally-fake-example-not-a-secret
EOF
  printf '%s\n' "$dir"
}

# Esegue il gate in DRY_RUN sul repo indicato, catturando stdout/stderr in
# file accanto per il debug in caso di assert falliti.
run_gate() {
  local repo_dir="$1"
  ( cd "$repo_dir" && DRY_RUN=1 bash "$GATE_SCRIPT" test-label ) \
    >"$repo_dir/gate.out" 2>"$repo_dir/gate.err"
}

dump_output() {
  local repo_dir="$1"
  echo "  --- stdout ---"
  sed 's/^/  /' "$repo_dir/gate.out" 2>/dev/null || true
  echo "  --- stderr ---"
  sed 's/^/  /' "$repo_dir/gate.err" 2>/dev/null || true
}

# --- Test A: repo pulito -> il gate DEVE passare ---
test_clean_repo_passes() {
  local repo
  repo="$(make_test_repo)"
  echo "hello world, nessun segreto qui" >"$repo/readme.txt"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "init"

  if run_gate "$repo"; then
    if grep -q "Gate segreti: OK" "$repo/gate.out"; then
      log_pass "repo pulito: il gate passa (exit 0)"
    else
      log_fail "repo pulito: exit 0 ma output inatteso"
      dump_output "$repo"
    fi
  else
    log_fail "repo pulito: il gate ha fallito ma doveva passare"
    dump_output "$repo"
  fi
  rm -rf "$repo"
}

# --- Test B: AWS key committata -> il gate DEVE fallire ---
test_aws_key_blocks() {
  local repo
  repo="$(make_test_repo)"
  echo "AWS_KEY=AKIAABCDEFGHIJKLMNOP" >"$repo/config.txt"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "init con AWS key"

  if run_gate "$repo"; then
    log_fail "AWS key: il gate ha PASSATO (exit 0) ma doveva fallire — REGRESSIONE gate-always-pass"
    dump_output "$repo"
  else
    log_pass "AWS key: il gate fallisce correttamente (exit != 0)"
  fi
  rm -rf "$repo"
}

# --- Test C: chiave privata OpenSSH committata -> il gate DEVE fallire ---
test_private_key_blocks() {
  local repo
  repo="$(make_test_repo)"
  cat >"$repo/id_ed25519" <<'EOF'
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBmYWtlZmFrZWZha2VmYWtlZmFrZWZha2VmYWtlZmFrZQAAAJBmYWtl
-----END OPENSSH PRIVATE KEY-----
EOF
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "init con chiave privata OpenSSH"

  if run_gate "$repo"; then
    log_fail "chiave privata OpenSSH: il gate ha PASSATO (exit 0) ma doveva fallire"
    dump_output "$repo"
  else
    log_pass "chiave privata OpenSSH: il gate fallisce correttamente (exit != 0)"
  fi
  rm -rf "$repo"
}

# --- Test D: segreto in file con byte binario spurio -> il gate DEVE fallire ---
# Riproduce l'HIGH "binary-file blind spot": un file con un singolo byte
# NUL in mezzo al testo viene classificato "binary" da grep di default
# (-I lo salterebbe silenziosamente). Il gate deve scansionarlo comunque.
test_secret_in_binary_classified_file_blocks() {
  local repo
  repo="$(make_test_repo)"
  printf 'AWS_KEY=AKIAABCDEFGHIJKLMNOP\n\x00\nmore text dopo il byte binario\n' >"$repo/weird.bin"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "init con segreto in file 'binario'"

  if run_gate "$repo"; then
    log_fail "segreto in file 'binario': il gate ha PASSATO (exit 0) ma doveva fallire — blind spot binario"
    dump_output "$repo"
  else
    log_pass "segreto in file 'binario': il gate fallisce correttamente (exit != 0)"
  fi
  rm -rf "$repo"
}

# --- Test E: blob base64 ad alta entropia (non lockfile) -> il gate DEVE fallire ---
test_base64_secret_blocks() {
  local repo secret
  repo="$(make_test_repo)"
  secret="$(python3 -c 'import base64, os; print(base64.b64encode(os.urandom(64)).decode())' 2>/dev/null \
    || openssl rand -base64 64 | tr -d '\n')"
  printf 'UPDATER_KEY=%s\n' "$secret" >"$repo/notes.txt"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "init con blob base64 ad alta entropia"

  if run_gate "$repo"; then
    log_fail "blob base64 ad alta entropia: il gate ha PASSATO (exit 0) ma doveva fallire"
    dump_output "$repo"
  else
    log_pass "blob base64 ad alta entropia: il gate fallisce correttamente (exit != 0)"
  fi
  rm -rf "$repo"
}

# --- Test F: hash esadecimale in Cargo.lock -> il gate DEVE passare (no falso positivo) ---
test_lockfile_hex_hash_does_not_block() {
  local repo hash
  repo="$(make_test_repo)"
  hash="$(printf 'x' | sha256sum | cut -d' ' -f1)"
  printf 'checksum = "%s%s"\n' "$hash" "$hash" >"$repo/Cargo.lock"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "init con Cargo.lock fittizio"

  if run_gate "$repo"; then
    log_pass "hash esadecimale in Cargo.lock: nessun falso positivo, il gate passa"
  else
    log_fail "hash esadecimale in Cargo.lock: falso positivo, il gate ha bloccato"
    dump_output "$repo"
  fi
  rm -rf "$repo"
}

test_clean_repo_passes
test_aws_key_blocks
test_private_key_blocks
test_secret_in_binary_classified_file_blocks
test_base64_secret_blocks
test_lockfile_hex_hash_does_not_block

echo ""
echo "Risultato: $PASS pass, $FAIL fail"
if [ "$FAIL" -ne 0 ]; then
  echo "REGRESSIONE RILEVATA nel gate segreti di publish-vetrina.sh." >&2
  exit 1
fi
exit 0
