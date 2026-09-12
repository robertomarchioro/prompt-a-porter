package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"reflect"
	"testing"
)

type casoConformita struct {
	Nome       string   `json:"nome"`
	Body       string   `json:"body"`
	Atteso     string   `json:"atteso"`
	Segnaposti []string `json:"segnaposti"`
}

// #643: la fixture è la fonte di verità condivisa con client TS, MCP e Rust.
func casiConformita(t *testing.T) []casoConformita {
	t.Helper()
	p := filepath.Join("..", "..", "packages", "shared-schema", "fixtures", "commenti-conformita.json")
	b, err := os.ReadFile(p)
	if err != nil {
		t.Fatalf("lettura fixture: %v", err)
	}
	var f struct {
		Casi []casoConformita `json:"casi"`
	}
	if err := json.Unmarshal(b, &f); err != nil {
		t.Fatalf("parsing fixture: %v", err)
	}
	if len(f.Casi) <= 10 {
		t.Fatalf("fixture sospettosamente corta: %d casi", len(f.Casi))
	}
	return f.Casi
}

func TestRimuoviCommentiConformita(t *testing.T) {
	for _, c := range casiConformita(t) {
		t.Run(c.Nome, func(t *testing.T) {
			if got := rimuoviCommenti(c.Body); got != c.Atteso {
				t.Errorf("rimuoviCommenti(%q) = %q, atteso %q", c.Body, got, c.Atteso)
			}
			got := estraiSegnaposti(c.Body)
			if len(got) == 0 && len(c.Segnaposti) == 0 {
				return
			}
			if !reflect.DeepEqual(got, c.Segnaposti) {
				t.Errorf("estraiSegnaposti(%q) = %v, atteso %v", c.Body, got, c.Segnaposti)
			}
		})
	}
}

func TestRimuoviCommentiIdempotente(t *testing.T) {
	for _, c := range casiConformita(t) {
		if got := rimuoviCommenti(c.Atteso); got != c.Atteso {
			t.Errorf("%s: non idempotente: %q → %q", c.Nome, c.Atteso, got)
		}
	}
}

func TestCompilaTogliCommenti(t *testing.T) {
	body := "{{!-- {{x}} qui non conta --}}\nCiao {{nome}} {{!-- inline --}}"
	got := compila(body, map[string]string{"x": "NO", "nome": "Anna"})
	if got != "Ciao Anna " {
		t.Errorf("compila = %q", got)
	}
}

func TestEspandiGlobaliNonTonaNeiCommenti(t *testing.T) {
	// L'ordine nel comando render è: commenti → globali → segnaposti.
	body := rimuoviCommenti("{{!-- {{global a}} --}}\nFirma: {{global a}}")
	got := espandiGlobali(body, map[string]string{"a": "Roberto"})
	if got != "Firma: Roberto" {
		t.Errorf("espandiGlobali = %q", got)
	}
}
