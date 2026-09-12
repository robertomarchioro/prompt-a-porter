package main

import (
	"regexp"
	"strings"
)

// Commenti inline nel body del prompt (issue #643): `{{!-- testo --}}`.
//
// Il commento è una nota dell'autore che non deve mai raggiungere il
// modello né gli appunti: `render` lo toglie PRIMA di globali e segnaposti.
// `get` invece mostra il sorgente, commenti inclusi.
//
// Semantica condivisa con client TS, MCP server e Rust, verificata dalla
// fixture `packages/shared-schema/fixtures/commenti-conformita.json`:
//   - unica forma `{{!-- … --}}`, chiude al primo `--}}`, multiriga;
//   - `{{! … }}` NON è un commento; un commento non chiuso resta inalterato;
//   - passo 1: si toglie il token; passo 2: ogni riga che conteneva un
//     commento e che dopo il passo 1 è vuota (solo spazi/tab/CR) sparisce con
//     il suo a-capo. Le righe già vuote in origine restano.

// (?s) perché il commento può andare a capo.
var reCommento = regexp.MustCompile(`(?s)\{\{!--.*?--\}\}`)

func rigaVuota(riga string) bool {
	return strings.Trim(riga, " \t\r") == ""
}

// rimuoviCommenti restituisce il testo senza i commenti `{{!-- … --}}`.
func rimuoviCommenti(testo string) string {
	intervalli := reCommento.FindAllStringIndex(testo, -1)
	if len(intervalli) == 0 {
		return testo
	}

	// Passo 1: togli i token, ricordando su quale riga dell'output cadevano.
	var senzaToken strings.Builder
	senzaToken.Grow(len(testo))
	righeToccate := map[int]bool{}
	rigaCorrente := 0
	cursore := 0
	for _, iv := range intervalli {
		pezzo := testo[cursore:iv[0]]
		rigaCorrente += strings.Count(pezzo, "\n")
		righeToccate[rigaCorrente] = true
		senzaToken.WriteString(pezzo)
		cursore = iv[1]
	}
	senzaToken.WriteString(testo[cursore:])

	// Passo 2: elimina le righe toccate rimaste vuote.
	righe := strings.Split(senzaToken.String(), "\n")
	out := make([]string, 0, len(righe))
	for i, riga := range righe {
		if righeToccate[i] && rigaVuota(riga) {
			continue
		}
		out = append(out, riga)
	}
	return strings.Join(out, "\n")
}
