package main

import (
	"database/sql"
	"strings"
	"testing"

	_ "modernc.org/sqlite"
)

func dbTest(t *testing.T) *sql.DB {
	t.Helper()
	db, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatalf("open: %v", err)
	}
	t.Cleanup(func() { _ = db.Close() })

	schema := `
		CREATE TABLE Prompts (
			Id TEXT PRIMARY KEY,
			WorkspaceId TEXT NOT NULL DEFAULT 'ws-personale',
			AuthorUserId TEXT NOT NULL DEFAULT 'usr-locale',
			Title TEXT NOT NULL,
			Description TEXT,
			Body TEXT NOT NULL,
			Visibility TEXT NOT NULL,
			TargetModel TEXT,
			IsFavorite INTEGER NOT NULL DEFAULT 0,
			UseCount INTEGER NOT NULL DEFAULT 0,
			LastUsedAt TEXT,
			Version INTEGER NOT NULL DEFAULT 1,
			CreatedAt TEXT NOT NULL,
			UpdatedAt TEXT NOT NULL,
			DeletedAt TEXT
		);
		CREATE TABLE Tags (
			Id TEXT PRIMARY KEY,
			WorkspaceId TEXT NOT NULL,
			Name TEXT NOT NULL,
			Color TEXT,
			CreatedAt TEXT NOT NULL,
			UpdatedAt TEXT NOT NULL,
			DeletedAt TEXT
		);
		CREATE TABLE PromptTags (
			PromptId TEXT NOT NULL,
			TagId TEXT NOT NULL,
			PRIMARY KEY (PromptId, TagId)
		);
		CREATE VIRTUAL TABLE PromptsFts USING fts5(
			PromptId UNINDEXED, Title, Description, Body, Tags
		);
	`
	if _, err := db.Exec(schema); err != nil {
		t.Fatalf("schema: %v", err)
	}
	return db
}

func TestSanitizzaFTS(t *testing.T) {
	tests := []struct {
		in, want string
	}{
		{"email", "email*"},
		{"  email  ", "email*"},
		{"email business", "email* business*"},
		{"hello, world!", "hello* world*"},
		{"", ""},
		{"!!!", ""},
		{"under_score", "under_score*"},
	}
	for _, tt := range tests {
		got := sanitizzaFTS(tt.in)
		if got != tt.want {
			t.Errorf("sanitizzaFTS(%q) = %q, want %q", tt.in, got, tt.want)
		}
	}
}

func TestEstraiSegnaposti(t *testing.T) {
	tests := []struct {
		body string
		want []string
	}{
		{"nessun segnaposto", nil},
		{"hello {{nome}}", []string{"nome"}},
		{"{{a}} {{b}} {{a}}", []string{"a", "b"}}, // dedup
		{"{{ spazi }}", []string{"spazi"}},
		{"{{a}}\n{{b}}\n{{c}}", []string{"a", "b", "c"}},
	}
	for _, tt := range tests {
		got := estraiSegnaposti(tt.body)
		if len(got) != len(tt.want) {
			t.Errorf("estraiSegnaposti(%q) len=%d want=%d (got %v)", tt.body, len(got), len(tt.want), got)
			continue
		}
		for i, name := range tt.want {
			if got[i] != name {
				t.Errorf("estraiSegnaposti(%q)[%d] = %q want %q", tt.body, i, got[i], name)
			}
		}
	}
}

func TestCompila(t *testing.T) {
	body := "Ciao {{nome}}, lavori in {{azienda}}?"
	vars := map[string]string{"nome": "Mario", "azienda": "Bluenergy"}
	got := compila(body, vars)
	want := "Ciao Mario, lavori in Bluenergy?"
	if got != want {
		t.Errorf("compila = %q, want %q", got, want)
	}
}

func TestCompilaSegnapostoNonCompilatoRestaTale(t *testing.T) {
	body := "{{a}} {{b}}"
	vars := map[string]string{"a": "X"}
	got := compila(body, vars)
	want := "X {{b}}"
	if got != want {
		t.Errorf("compila = %q, want %q", got, want)
	}
}

func TestCompilaValoreVuotoRestaSegnaposto(t *testing.T) {
	body := "{{x}}"
	vars := map[string]string{"x": "   "}
	got := compila(body, vars)
	if got != "{{x}}" {
		t.Errorf("compila con valore whitespace = %q, want {{x}}", got)
	}
}

func TestFormatPromptsTable(t *testing.T) {
	tm := "claude-sonnet"
	prompts := []Prompt{
		{ID: "prm-1", Title: "Test", Visibility: "private", TargetModel: &tm, UseCount: 5, Tags: []string{"bug", "eng"}},
	}
	out, err := formatPrompts(prompts, "table")
	if err != nil {
		t.Fatalf("formatPrompts: %v", err)
	}
	if !strings.Contains(out, "ID") || !strings.Contains(out, "TITLE") || !strings.Contains(out, "prm-1") {
		t.Errorf("table output mancante header o riga: %q", out)
	}
}

func TestFormatPromptsJSON(t *testing.T) {
	prompts := []Prompt{{ID: "prm-1", Title: "X", Visibility: "private"}}
	out, err := formatPrompts(prompts, "json")
	if err != nil {
		t.Fatalf("formatPrompts: %v", err)
	}
	if !strings.Contains(out, `"id": "prm-1"`) {
		t.Errorf("json output mancante id: %q", out)
	}
}

func TestFormatPromptsYAML(t *testing.T) {
	prompts := []Prompt{{ID: "prm-1", Title: "X", Visibility: "private"}}
	out, err := formatPrompts(prompts, "yaml")
	if err != nil {
		t.Fatalf("formatPrompts: %v", err)
	}
	if !strings.Contains(out, "id: prm-1") {
		t.Errorf("yaml output mancante id: %q", out)
	}
}

func TestFormatPromptsPlain(t *testing.T) {
	prompts := []Prompt{
		{ID: "prm-1", Title: "X", Visibility: "private"},
		{ID: "prm-2", Title: "Y", Visibility: "private"},
	}
	out, err := formatPrompts(prompts, "plain")
	if err != nil {
		t.Fatalf("formatPrompts: %v", err)
	}
	want := "prm-1\tX\nprm-2\tY\n"
	if out != want {
		t.Errorf("plain output = %q, want %q", out, want)
	}
}

func TestFormatPromptsInvalido(t *testing.T) {
	_, err := formatPrompts(nil, "csv")
	if err == nil {
		t.Errorf("formato csv deve dare errore")
	}
}

func TestSearchEdgeCases(t *testing.T) {
	db := dbTest(t)

	// Vuoto: nessun prompt → nessun risultato
	prompts, err := search(db, "", 10, "", "")
	if err != nil {
		t.Fatalf("search vuoto: %v", err)
	}
	if len(prompts) != 0 {
		t.Errorf("expected 0 results, got %d", len(prompts))
	}

	// Aggiungi un prompt
	_, err = db.Exec(`INSERT INTO Prompts (Id, Title, Body, Visibility, CreatedAt, UpdatedAt)
		VALUES ('prm-1', 'Email marketing', 'Scrivi una email', 'private',
		        '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')`)
	if err != nil {
		t.Fatalf("insert: %v", err)
	}
	_, err = db.Exec(`INSERT INTO PromptsFts (PromptId, Title, Description, Body, Tags)
		VALUES ('prm-1', 'Email marketing', '', 'Scrivi una email', '')`)
	if err != nil {
		t.Fatalf("insert fts: %v", err)
	}

	// Search con query → trova
	prompts, err = search(db, "email", 10, "", "")
	if err != nil {
		t.Fatalf("search email: %v", err)
	}
	if len(prompts) != 1 || prompts[0].ID != "prm-1" {
		t.Errorf("expected prm-1, got %v", prompts)
	}

	// Search vuoto → trova (recenti)
	prompts, err = search(db, "", 10, "", "")
	if err != nil {
		t.Fatalf("search recenti: %v", err)
	}
	if len(prompts) != 1 {
		t.Errorf("expected 1 result, got %d", len(prompts))
	}
}

func TestGetNotFound(t *testing.T) {
	db := dbTest(t)
	_, err := get(db, "prm-inesistente")
	if err == nil {
		t.Errorf("get di id inesistente deve dare errore")
	}
}

func TestDefaultVaultPathNonVuoto(t *testing.T) {
	p := defaultVaultPath()
	if p == "" || !strings.HasSuffix(p, "pap-vault.db") {
		t.Errorf("defaultVaultPath inatteso: %q", p)
	}
}

func TestTruncate(t *testing.T) {
	if truncate("abc", 10) != "abc" {
		t.Errorf("truncate non dovrebbe modificare stringhe corte")
	}
	if !strings.HasSuffix(truncate("abcdefghijklmno", 10), "…") {
		t.Errorf("truncate dovrebbe finire con ellissi")
	}
}
