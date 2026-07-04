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

func TestTagsFor(t *testing.T) {
	db := dbTest(t)

	_, err := db.Exec(`INSERT INTO Prompts (Id, Title, Body, Visibility, CreatedAt, UpdatedAt)
		VALUES ('prm-1', 'P', 'B', 'private', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')`)
	if err != nil {
		t.Fatalf("insert prompt: %v", err)
	}
	_, err = db.Exec(`INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
		VALUES ('tag-bug', 'ws-personale', 'bug', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
		       ('tag-eng', 'ws-personale', 'eng', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
		       ('tag-del', 'ws-personale', 'cancellato', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')`)
	if err != nil {
		t.Fatalf("insert tags: %v", err)
	}
	// Soft-delete del tag "cancellato" — non deve apparire
	_, err = db.Exec(`UPDATE Tags SET DeletedAt = '2026-01-02T00:00:00Z' WHERE Id = 'tag-del'`)
	if err != nil {
		t.Fatalf("soft delete: %v", err)
	}
	_, err = db.Exec(`INSERT INTO PromptTags (PromptId, TagId) VALUES
		('prm-1', 'tag-bug'), ('prm-1', 'tag-eng'), ('prm-1', 'tag-del')`)
	if err != nil {
		t.Fatalf("insert prompttags: %v", err)
	}

	tags, err := tagsFor(db, "prm-1")
	if err != nil {
		t.Fatalf("tagsFor: %v", err)
	}
	// Atteso: ordinato ASC per Name, esclude soft-deleted
	want := []string{"bug", "eng"}
	if len(tags) != len(want) {
		t.Fatalf("tagsFor len = %d, want %d (got %v)", len(tags), len(want), tags)
	}
	for i, name := range want {
		if tags[i] != name {
			t.Errorf("tagsFor[%d] = %q, want %q", i, tags[i], name)
		}
	}

	// Prompt senza tag → slice vuoto, no errore
	_, err = db.Exec(`INSERT INTO Prompts (Id, Title, Body, Visibility, CreatedAt, UpdatedAt)
		VALUES ('prm-2', 'P2', 'B2', 'private', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')`)
	if err != nil {
		t.Fatalf("insert prompt 2: %v", err)
	}
	tags, err = tagsFor(db, "prm-2")
	if err != nil {
		t.Fatalf("tagsFor vuoto: %v", err)
	}
	if len(tags) != 0 {
		t.Errorf("tagsFor su prompt senza tag = %v, want []", tags)
	}
}

func TestRecent(t *testing.T) {
	db := dbTest(t)

	// 3 prompt con LastUsedAt diversi (uno NULL, fallback a UpdatedAt)
	_, err := db.Exec(`INSERT INTO Prompts (Id, Title, Body, Visibility, LastUsedAt, CreatedAt, UpdatedAt) VALUES
		('prm-old',    'Vecchio',    'B', 'private', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
		('prm-recent', 'Recente',    'B', 'private', '2026-03-01T00:00:00Z', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
		('prm-fallbk', 'Senza last', 'B', 'private', NULL,                   '2026-01-01T00:00:00Z', '2026-02-01T00:00:00Z')`)
	if err != nil {
		t.Fatalf("insert: %v", err)
	}
	// Soft-delete: non deve apparire
	_, err = db.Exec(`INSERT INTO Prompts (Id, Title, Body, Visibility, CreatedAt, UpdatedAt, DeletedAt) VALUES
		('prm-del', 'Cancellato', 'B', 'private', '2026-01-01T00:00:00Z', '2026-04-01T00:00:00Z', '2026-04-01T00:00:00Z')`)
	if err != nil {
		t.Fatalf("insert deleted: %v", err)
	}
	// Tag su prm-recent per coprire anche tagsFor
	_, err = db.Exec(`INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
		VALUES ('tag-1', 'ws-personale', 'urgente', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')`)
	if err != nil {
		t.Fatalf("insert tag: %v", err)
	}
	_, err = db.Exec(`INSERT INTO PromptTags (PromptId, TagId) VALUES ('prm-recent', 'tag-1')`)
	if err != nil {
		t.Fatalf("insert prompttag: %v", err)
	}

	prompts, err := recent(db, 10)
	if err != nil {
		t.Fatalf("recent: %v", err)
	}
	if len(prompts) != 3 {
		t.Fatalf("recent len = %d, want 3 (esclude soft-deleted)", len(prompts))
	}
	// Ordine atteso: prm-recent (LastUsedAt 2026-03), prm-fallbk (UpdatedAt 2026-02), prm-old (LastUsedAt 2026-01)
	wantOrder := []string{"prm-recent", "prm-fallbk", "prm-old"}
	for i, id := range wantOrder {
		if prompts[i].ID != id {
			t.Errorf("recent[%d].ID = %q, want %q", i, prompts[i].ID, id)
		}
	}
	// Tag deve essere popolato
	if len(prompts[0].Tags) != 1 || prompts[0].Tags[0] != "urgente" {
		t.Errorf("recent[0].Tags = %v, want [urgente]", prompts[0].Tags)
	}

	// Limit clamp: < 1 → 10, > 100 → 100. Test su valore basso.
	prompts, err = recent(db, 1)
	if err != nil {
		t.Fatalf("recent limit=1: %v", err)
	}
	if len(prompts) != 1 {
		t.Errorf("recent con limit=1 deve restituire 1 prompt, got %d", len(prompts))
	}
}

func TestFormatPrompt(t *testing.T) {
	desc := "una descrizione"
	tm := "claude-sonnet"
	last := "2026-03-01T00:00:00Z"
	p := Prompt{
		ID: "prm-1", Title: "Test", Description: &desc, Body: "Ciao {{nome}}",
		Visibility: "private", TargetModel: &tm, UseCount: 5, LastUsedAt: &last,
		Version: 2, Tags: []string{"bug", "eng"},
	}

	// Table format
	out, err := formatPrompt(p, "table")
	if err != nil {
		t.Fatalf("formatPrompt table: %v", err)
	}
	for _, want := range []string{"prm-1", "Test", "una descrizione", "claude-sonnet", "bug, eng", "nome", "Ciao {{nome}}"} {
		if !strings.Contains(out, want) {
			t.Errorf("table output mancante %q in: %s", want, out)
		}
	}

	// Default (vuoto) = table
	outDefault, err := formatPrompt(p, "")
	if err != nil {
		t.Fatalf("formatPrompt default: %v", err)
	}
	if outDefault != out {
		t.Errorf("formatPrompt(\"\") deve coincidere con \"table\"")
	}

	// JSON
	out, err = formatPrompt(p, "json")
	if err != nil {
		t.Fatalf("formatPrompt json: %v", err)
	}
	if !strings.Contains(out, `"id": "prm-1"`) || !strings.Contains(out, `"title": "Test"`) {
		t.Errorf("json output sbagliato: %s", out)
	}

	// YAML
	out, err = formatPrompt(p, "yaml")
	if err != nil {
		t.Fatalf("formatPrompt yaml: %v", err)
	}
	if !strings.Contains(out, "id: prm-1") || !strings.Contains(out, "title: Test") {
		t.Errorf("yaml output sbagliato: %s", out)
	}

	// Plain → solo body
	out, err = formatPrompt(p, "plain")
	if err != nil {
		t.Fatalf("formatPrompt plain: %v", err)
	}
	if out != "Ciao {{nome}}" {
		t.Errorf("plain output = %q, want body", out)
	}

	// Formato non supportato → errore
	_, err = formatPrompt(p, "csv")
	if err == nil {
		t.Errorf("formato csv deve dare errore")
	}

	// Prompt minimo (no description, no target, no last used, no tags) — coverage rami else
	pMin := Prompt{ID: "prm-2", Title: "Min", Body: "body", Visibility: "private", Version: 1}
	out, err = formatPrompt(pMin, "table")
	if err != nil {
		t.Fatalf("formatPrompt minimo: %v", err)
	}
	if !strings.Contains(out, "prm-2") || strings.Contains(out, "Descrizione:") ||
		strings.Contains(out, "Target:") || strings.Contains(out, "Tag:") {
		t.Errorf("table minimo non dovrebbe avere campi opzionali: %s", out)
	}
}

// ─── #render globali + import (fix v0.8.x) ───

func TestEspandiGlobali(t *testing.T) {
	globals := map[string]string{"autore": "Mario Rossi", "vuoto": "  "}
	body := "Di {{global autore}}. Manca {{global ignoto}}. Vuoto {{global vuoto}}. Var {{nome}}."
	got := espandiGlobali(body, globals)
	want := "Di Mario Rossi. Manca {{global ignoto}}. Vuoto {{global vuoto}}. Var {{nome}}."
	if got != want {
		t.Errorf("espandiGlobali:\n got=%q\nwant=%q", got, want)
	}
}

func TestCaricaGlobali(t *testing.T) {
	db := dbTest(t)
	// La tabella non è nello schema base: la creiamo qui (come V015).
	if _, err := db.Exec(`CREATE TABLE GlobalPlaceholders (Name TEXT PRIMARY KEY, Value TEXT NOT NULL, UpdatedAt TEXT NOT NULL DEFAULT (datetime('now')))`); err != nil {
		t.Fatalf("create: %v", err)
	}
	if _, err := db.Exec(`INSERT INTO GlobalPlaceholders (Name, Value) VALUES ('autore','Anna'),('team','PaP')`); err != nil {
		t.Fatalf("insert: %v", err)
	}
	g := caricaGlobali(db)
	if g["autore"] != "Anna" || g["team"] != "PaP" || len(g) != 2 {
		t.Errorf("caricaGlobali = %v", g)
	}
}

func TestCaricaGlobaliTabellaAssente(t *testing.T) {
	db := dbTest(t) // schema base senza GlobalPlaceholders
	g := caricaGlobali(db)
	if len(g) != 0 {
		t.Errorf("vault senza tabella globali deve dare mappa vuota, got %v", g)
	}
}

func TestNomiUnici(t *testing.T) {
	body := `Import {{import "intro/base"}} e {{import "intro/base"}} e {{import "altro" with k=v}}. Global {{global x}}.`
	imp := nomiUnici(reImport, body)
	if len(imp) != 2 || imp[0] != "intro/base" || imp[1] != "altro" {
		t.Errorf("import paths = %v", imp)
	}
	glob := nomiUnici(reGlobal, body)
	if len(glob) != 1 || glob[0] != "x" {
		t.Errorf("global names = %v", glob)
	}
}
