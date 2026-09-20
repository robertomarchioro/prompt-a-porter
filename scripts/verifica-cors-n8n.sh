#!/usr/bin/env bash
# Verifica dall'esterno che la patch CORS su nginx (CT 100) davanti a n8n sia
# applicata per i webhook del form «Mandami il link» (istruzioni-sviluppo-landing.md §5.2).
#
# Cosa controlla, per ogni percorso webhook:
#   1. preflight OPTIONS con Origin www → 2xx, ACAO uguale all'origin, POST e content-type ammessi
#   2. preflight con origin apex e con origin estraneo → l'origin NON deve essere riflesso
#   3. richiesta reale (POST) con Origin www → ACAO presente anche se n8n risponde 404
#      (add_header senza `always` sparisce sugli errori: il form fallirebbe a webhook spento)
#
# Uso:  scripts/verifica-cors-n8n.sh https://n8n.esempio.it [percorso ...]
#       percorsi di default: /webhook/pap-download /webhook-test/pap-download /webhook/pap-challenge
# Esce 0 se tutti i controlli passano, 1 altrimenti. Nessun dato personale viene inviato.

# niente -e: i grep senza match (header assente) sono esiti attesi, non errori
set -uo pipefail

readonly ORIGIN_OK="https://www.promptaporter.it"
readonly ORIGIN_APEX="https://promptaporter.it"
readonly ORIGIN_ESTRANEO="https://esempio-estraneo.invalid"
readonly PERCORSI_DEFAULT=(/webhook/pap-download /webhook-test/pap-download /webhook/pap-challenge)
readonly TIMEOUT_SECONDI=10

if [[ $# -lt 1 ]]; then
  echo "Uso: $0 https://host-n8n [percorso ...]" >&2
  exit 2
fi

readonly BASE="${1%/}"
shift
PERCORSI=("$@")
[[ ${#PERCORSI[@]} -eq 0 ]] && PERCORSI=("${PERCORSI_DEFAULT[@]}")

errori=0
avvisi=0

ok()      { echo "  ✅ $1"; }
ko()      { echo "  ❌ $1"; errori=$((errori + 1)); }
avviso()  { echo "  ⚠️  $1"; avvisi=$((avvisi + 1)); }

# Stampa gli header di risposta (prima riga di stato inclusa), senza corpo.
# Ultimo argomento: URL; gli altri: opzioni curl aggiuntive.
intestazioni() {
  curl --silent --show-error --max-time "$TIMEOUT_SECONDI" --dump-header - --output /dev/null "$@" \
    || echo "HTTP/1.1 000 CONNESSIONE-FALLITA"
}

# Estrae il valore di un header (case-insensitive), senza CR finale.
valore_header() {
  local nome="$1"
  { grep -i "^${nome}:" || true; } | head -n 1 | cut -d: -f2- | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | tr -d '\r'
}

codice_stato() {
  { grep -E '^HTTP/' || true; } | tail -n 1 | awk '{print $2}'
}

contiene_token() {
  # $1 = lista separata da virgole, $2 = token cercato (case-insensitive)
  echo "$1" | tr '[:upper:]' '[:lower:]' | tr ',' '\n' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | grep -qx "$(echo "$2" | tr '[:upper:]' '[:lower:]')"
}

verifica_preflight_ammesso() {
  local url="$1"
  local risposta stato acao metodi header vary
  risposta=$(intestazioni -X OPTIONS \
    -H "Origin: $ORIGIN_OK" \
    -H "Access-Control-Request-Method: POST" \
    -H "Access-Control-Request-Headers: content-type" \
    "$url")
  stato=$(echo "$risposta" | codice_stato)
  acao=$(echo "$risposta" | valore_header "Access-Control-Allow-Origin")
  metodi=$(echo "$risposta" | valore_header "Access-Control-Allow-Methods")
  header=$(echo "$risposta" | valore_header "Access-Control-Allow-Headers")
  vary=$(echo "$risposta" | valore_header "Vary")

  if [[ "$stato" =~ ^2 ]]; then ok "preflight OPTIONS → $stato"; else ko "preflight OPTIONS → $stato (atteso 2xx)"; fi

  if [[ "$acao" == "$ORIGIN_OK" ]]; then
    ok "Access-Control-Allow-Origin = $acao"
  elif [[ "$acao" == "*" ]]; then
    avviso "Access-Control-Allow-Origin = * (funziona, ma la regex è più larga del previsto)"
  else
    ko "Access-Control-Allow-Origin = '${acao:-assente}' (atteso $ORIGIN_OK)"
  fi

  if contiene_token "$metodi" POST; then ok "Allow-Methods ammette POST ($metodi)"; else ko "Allow-Methods non ammette POST ('${metodi:-assente}')"; fi
  if contiene_token "$header" content-type; then ok "Allow-Headers ammette content-type"; else ko "Allow-Headers non ammette content-type ('${header:-assente}')"; fi
  if echo "$vary" | grep -qi origin; then ok "Vary: Origin presente"; else avviso "Vary: Origin assente (rischio cache condivisa fra origin)"; fi
}

verifica_origin_rifiutato() {
  local url="$1" origin="$2" etichetta="$3"
  local acao
  acao=$(intestazioni -X OPTIONS -H "Origin: $origin" -H "Access-Control-Request-Method: POST" "$url" \
    | valore_header "Access-Control-Allow-Origin")
  if [[ -z "$acao" ]]; then
    ok "origin $etichetta ($origin) → nessun ACAO"
  elif [[ "$acao" == "$ORIGIN_OK" ]]; then
    ok "origin $etichetta → ACAO resta $ORIGIN_OK (non riflesso)"
  else
    ko "origin $etichetta → ACAO = '$acao' (riflesso: regex troppo permissiva)"
  fi
}

verifica_richiesta_reale() {
  local url="$1"
  local risposta stato acao
  risposta=$(intestazioni -X POST -H "Origin: $ORIGIN_OK" -H "Content-Type: application/json" --data '{}' "$url")
  stato=$(echo "$risposta" | codice_stato)
  acao=$(echo "$risposta" | valore_header "Access-Control-Allow-Origin")
  if [[ -n "$acao" ]]; then
    ok "POST reale → $stato con ACAO presente ($acao)"
  elif [[ "$stato" =~ ^(404|405|5) ]]; then
    ko "POST reale → $stato SENZA ACAO: add_header senza 'always' — a webhook spento il browser vede un errore CORS"
  else
    ko "POST reale → $stato senza ACAO"
  fi
}

echo "Verifica CORS su $BASE per l'origin $ORIGIN_OK"
for percorso in "${PERCORSI[@]}"; do
  url="${BASE}${percorso}"
  echo
  echo "▶ $percorso"
  verifica_preflight_ammesso "$url"
  verifica_origin_rifiutato "$url" "$ORIGIN_APEX" "apex"
  verifica_origin_rifiutato "$url" "$ORIGIN_ESTRANEO" "estraneo"
  verifica_richiesta_reale "$url"
done

echo
echo "Esito: $errori errori, $avvisi avvisi"
[[ $errori -eq 0 ]]
