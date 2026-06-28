// Package main è il CLI `pap` di Prompt a Porter.
// Read-only su vault locale non cifrato; comandi: version, search, get,
// recent, render. Completion automatico via cobra.
package main

import (
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"
	"text/tabwriter"

	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
	_ "modernc.org/sqlite"
)

const (
	cliVersion    = "0.1.0"
	identifier    = "com.pap.client"
	vaultFilename = "pap-vault.db"
)

// ─── Tipi ───

type Prompt struct {
	ID          string   `json:"id" yaml:"id"`
	Title       string   `json:"title" yaml:"title"`
	Description *string  `json:"description,omitempty" yaml:"description,omitempty"`
	Body        string   `json:"body,omitempty" yaml:"body,omitempty"`
	Visibility  string   `json:"visibility" yaml:"visibility"`
	TargetModel *string  `json:"target_model,omitempty" yaml:"target_model,omitempty"`
	IsFavorite  bool     `json:"is_favorite" yaml:"is_favorite"`
	UseCount    int64    `json:"use_count" yaml:"use_count"`
	LastUsedAt  *string  `json:"last_used_at,omitempty" yaml:"last_used_at,omitempty"`
	Version     int64    `json:"version" yaml:"version"`
	CreatedAt   string   `json:"created_at" yaml:"created_at"`
	UpdatedAt   string   `json:"updated_at" yaml:"updated_at"`
	Tags        []string `json:"tags,omitempty" yaml:"tags,omitempty"`
}

// ─── Vault discovery ───

func defaultVaultPath() string {
	home, err := os.UserHomeDir()
	if err != nil {
		home = ""
	}
	switch runtime.GOOS {
	case "darwin":
		return filepath.Join(home, "Library", "Application Support", identifier, vaultFilename)
	case "windows":
		appData := os.Getenv("APPDATA")
		if appData == "" {
			appData = filepath.Join(home, "AppData", "Roaming")
		}
		return filepath.Join(appData, identifier, vaultFilename)
	default:
		xdg := os.Getenv("XDG_DATA_HOME")
		if xdg == "" {
			xdg = filepath.Join(home, ".local", "share")
		}
		return filepath.Join(xdg, identifier, vaultFilename)
	}
}

func vaultPath() string {
	if p := os.Getenv("PAP_VAULT_PATH"); p != "" {
		return p
	}
	return defaultVaultPath()
}

func openVault(path string) (*sql.DB, error) {
	if _, err := os.Stat(path); errors.Is(err, os.ErrNotExist) {
		return nil, fmt.Errorf("vault non trovato: %s\nImposta PAP_VAULT_PATH o crea il vault dal client desktop", path)
	}
	// modernc/sqlite usa "sqlite" come driver name. Mode read-only via DSN query.
	dsn := fmt.Sprintf("file:%s?mode=ro", path)
	db, err := sql.Open("sqlite", dsn)
	if err != nil {
		return nil, fmt.Errorf("apertura vault: %w", err)
	}
	if err := db.Ping(); err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("ping vault: %w", err)
	}
	return db, nil
}

// ─── Query helpers ───

func sanitizzaFTS(query string) string {
	parts := strings.Fields(query)
	out := make([]string, 0, len(parts))
	re := regexp.MustCompile(`[^\p{L}\p{N}_]`)
	for _, w := range parts {
		clean := re.ReplaceAllString(w, "")
		if clean != "" {
			out = append(out, clean+"*")
		}
	}
	return strings.Join(out, " ")
}

func tagsFor(db *sql.DB, promptID string) ([]string, error) {
	rows, err := db.Query(`
		SELECT t.Name FROM Tags t
		JOIN PromptTags pt ON pt.TagId = t.Id
		WHERE pt.PromptId = ? AND t.DeletedAt IS NULL
		ORDER BY t.Name ASC`, promptID)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()
	var tags []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return nil, err
		}
		tags = append(tags, name)
	}
	return tags, rows.Err()
}

func scanPrompts(rows *sql.Rows) ([]Prompt, error) {
	var out []Prompt
	for rows.Next() {
		var p Prompt
		var fav int64
		if err := rows.Scan(
			&p.ID, &p.Title, &p.Description, &p.Body, &p.Visibility,
			&p.TargetModel, &fav, &p.UseCount, &p.LastUsedAt, &p.Version,
			&p.CreatedAt, &p.UpdatedAt,
		); err != nil {
			return nil, err
		}
		p.IsFavorite = fav != 0
		out = append(out, p)
	}
	return out, rows.Err()
}

func search(db *sql.DB, query string, limit int, targetModel, tagFilter string) ([]Prompt, error) {
	if limit < 1 {
		limit = 10
	}
	if limit > 100 {
		limit = 100
	}

	var rows *sql.Rows
	var err error
	if strings.TrimSpace(query) == "" {
		args := []any{}
		sql := `SELECT Id, Title, Description, Body, Visibility, TargetModel,
		               IsFavorite, UseCount, LastUsedAt, Version, CreatedAt, UpdatedAt
		        FROM Prompts WHERE DeletedAt IS NULL`
		if targetModel != "" {
			sql += ` AND TargetModel = ?`
			args = append(args, targetModel)
		}
		sql += ` ORDER BY COALESCE(LastUsedAt, UpdatedAt) DESC LIMIT ?`
		args = append(args, limit)
		rows, err = db.Query(sql, args...)
	} else {
		fts := sanitizzaFTS(query)
		if fts == "" {
			return nil, nil
		}
		args := []any{fts}
		sql := `SELECT p.Id, p.Title, p.Description, p.Body, p.Visibility, p.TargetModel,
		               p.IsFavorite, p.UseCount, p.LastUsedAt, p.Version, p.CreatedAt, p.UpdatedAt
		        FROM PromptsFts f
		        JOIN Prompts p ON f.PromptId = p.Id
		        WHERE PromptsFts MATCH ? AND p.DeletedAt IS NULL`
		if targetModel != "" {
			sql += ` AND p.TargetModel = ?`
			args = append(args, targetModel)
		}
		sql += ` ORDER BY rank LIMIT ?`
		args = append(args, limit)
		rows, err = db.Query(sql, args...)
	}
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()

	prompts, err := scanPrompts(rows)
	if err != nil {
		return nil, err
	}

	// Filtro tag (post-query, MVP semplice).
	if tagFilter != "" {
		var filtered []Prompt
		for _, p := range prompts {
			tags, err := tagsFor(db, p.ID)
			if err != nil {
				return nil, err
			}
			for _, t := range tags {
				if t == tagFilter {
					p.Tags = tags
					filtered = append(filtered, p)
					break
				}
			}
		}
		return filtered, nil
	}

	// Popola tag per tutti.
	for i := range prompts {
		tags, err := tagsFor(db, prompts[i].ID)
		if err != nil {
			return nil, err
		}
		prompts[i].Tags = tags
	}
	return prompts, nil
}

func get(db *sql.DB, id string) (*Prompt, error) {
	rows, err := db.Query(`
		SELECT Id, Title, Description, Body, Visibility, TargetModel,
		       IsFavorite, UseCount, LastUsedAt, Version, CreatedAt, UpdatedAt
		FROM Prompts WHERE Id = ? AND DeletedAt IS NULL`, id)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()
	prompts, err := scanPrompts(rows)
	if err != nil {
		return nil, err
	}
	if len(prompts) == 0 {
		return nil, fmt.Errorf("prompt %s non trovato", id)
	}
	tags, err := tagsFor(db, id)
	if err != nil {
		return nil, err
	}
	prompts[0].Tags = tags
	return &prompts[0], nil
}

func recent(db *sql.DB, limit int) ([]Prompt, error) {
	if limit < 1 {
		limit = 10
	}
	if limit > 100 {
		limit = 100
	}
	rows, err := db.Query(`
		SELECT Id, Title, Description, Body, Visibility, TargetModel,
		       IsFavorite, UseCount, LastUsedAt, Version, CreatedAt, UpdatedAt
		FROM Prompts WHERE DeletedAt IS NULL
		ORDER BY COALESCE(LastUsedAt, UpdatedAt) DESC LIMIT ?`, limit)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()
	prompts, err := scanPrompts(rows)
	if err != nil {
		return nil, err
	}
	for i := range prompts {
		tags, err := tagsFor(db, prompts[i].ID)
		if err != nil {
			return nil, err
		}
		prompts[i].Tags = tags
	}
	return prompts, nil
}

// ─── Template render ───

var reSegnaposto = regexp.MustCompile(`\{\{\s*(\w+)\s*\}\}`)

func compila(body string, vars map[string]string) string {
	return reSegnaposto.ReplaceAllStringFunc(body, func(match string) string {
		sub := reSegnaposto.FindStringSubmatch(match)
		if len(sub) < 2 {
			return match
		}
		v, ok := vars[sub[1]]
		if !ok || strings.TrimSpace(v) == "" {
			return match
		}
		return v
	})
}

func estraiSegnaposti(body string) []string {
	matches := reSegnaposto.FindAllStringSubmatch(body, -1)
	seen := make(map[string]bool, len(matches))
	out := make([]string, 0, len(matches))
	for _, m := range matches {
		if !seen[m[1]] {
			seen[m[1]] = true
			out = append(out, m[1])
		}
	}
	return out
}

// ─── Output formatters ───

func formatPrompts(prompts []Prompt, format string) (string, error) {
	switch format {
	case "json":
		b, err := json.MarshalIndent(prompts, "", "  ")
		if err != nil {
			return "", err
		}
		return string(b), nil
	case "yaml":
		b, err := yaml.Marshal(prompts)
		if err != nil {
			return "", err
		}
		return string(b), nil
	case "plain":
		var sb strings.Builder
		for _, p := range prompts {
			sb.WriteString(p.ID)
			sb.WriteByte('\t')
			sb.WriteString(p.Title)
			sb.WriteByte('\n')
		}
		return sb.String(), nil
	case "table", "":
		var sb strings.Builder
		w := tabwriter.NewWriter(&sb, 0, 0, 2, ' ', 0)
		_, _ = fmt.Fprintln(w, "ID\tTITLE\tVISIBILITY\tTARGET\tUSE\tTAGS")
		for _, p := range prompts {
			tm := "-"
			if p.TargetModel != nil {
				tm = *p.TargetModel
			}
			tags := "-"
			if len(p.Tags) > 0 {
				tags = strings.Join(p.Tags, ",")
			}
			_, _ = fmt.Fprintf(w, "%s\t%s\t%s\t%s\t%d\t%s\n",
				p.ID, truncate(p.Title, 40), p.Visibility, tm, p.UseCount, tags)
		}
		_ = w.Flush()
		return sb.String(), nil
	default:
		return "", fmt.Errorf("formato non supportato: %s (attesi: table, json, yaml, plain)", format)
	}
}

func formatPrompt(p Prompt, format string) (string, error) {
	// Per il dettaglio singolo, JSON/YAML output l'oggetto direttamente.
	switch format {
	case "json":
		b, err := json.MarshalIndent(p, "", "  ")
		if err != nil {
			return "", err
		}
		return string(b), nil
	case "yaml":
		b, err := yaml.Marshal(p)
		if err != nil {
			return "", err
		}
		return string(b), nil
	case "plain":
		return p.Body, nil
	case "table", "":
		var sb strings.Builder
		fmt.Fprintf(&sb, "ID:          %s\n", p.ID)
		fmt.Fprintf(&sb, "Titolo:      %s\n", p.Title)
		if p.Description != nil && *p.Description != "" {
			fmt.Fprintf(&sb, "Descrizione: %s\n", *p.Description)
		}
		fmt.Fprintf(&sb, "Visibilità:  %s\n", p.Visibility)
		if p.TargetModel != nil {
			fmt.Fprintf(&sb, "Target:      %s\n", *p.TargetModel)
		}
		fmt.Fprintf(&sb, "Versione:    %d\n", p.Version)
		fmt.Fprintf(&sb, "Use count:   %d\n", p.UseCount)
		if p.LastUsedAt != nil {
			fmt.Fprintf(&sb, "Ultimo uso:  %s\n", *p.LastUsedAt)
		}
		if len(p.Tags) > 0 {
			fmt.Fprintf(&sb, "Tag:         %s\n", strings.Join(p.Tags, ", "))
		}
		segn := estraiSegnaposti(p.Body)
		if len(segn) > 0 {
			fmt.Fprintf(&sb, "Segnaposti:  %s\n", strings.Join(segn, ", "))
		}
		fmt.Fprintln(&sb, "---")
		fmt.Fprintln(&sb, p.Body)
		return sb.String(), nil
	default:
		return "", fmt.Errorf("formato non supportato: %s (attesi: table, json, yaml, plain)", format)
	}
}

func truncate(s string, n int) string {
	if len(s) <= n {
		return s
	}
	return s[:n-1] + "…"
}

// ─── Cobra commands ───

var rootCmd = &cobra.Command{
	Use:           "pap",
	Short:         "Prompt a Porter — CLI per il vault locale",
	Long:          "CLI di Prompt a Porter per cercare, leggere e compilare prompt dal vault locale.\nLegge in read-only il file pap-vault.db; per modificare i prompt usa il client desktop.",
	SilenceUsage:  true,
	SilenceErrors: false,
}

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Mostra versione e path del vault risolto",
	RunE: func(cmd *cobra.Command, args []string) error {
		_, _ = fmt.Fprintf(cmd.OutOrStdout(), "pap %s\nvault: %s\n", cliVersion, vaultPath())
		return nil
	},
}

var searchCmd = &cobra.Command{
	Use:   "search [query]",
	Short: "Cerca prompt via FTS5 con filtri opzionali",
	Args:  cobra.MaximumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		query := ""
		if len(args) == 1 {
			query = args[0]
		}
		limit, _ := cmd.Flags().GetInt("limit")
		target, _ := cmd.Flags().GetString("target")
		tag, _ := cmd.Flags().GetString("tag")
		format, _ := cmd.Flags().GetString("format")

		db, err := openVault(vaultPath())
		if err != nil {
			return err
		}
		defer func() { _ = db.Close() }()

		prompts, err := search(db, query, limit, target, tag)
		if err != nil {
			return err
		}
		out, err := formatPrompts(prompts, format)
		if err != nil {
			return err
		}
		_, _ = fmt.Fprint(cmd.OutOrStdout(), out)
		return nil
	},
}

var getCmd = &cobra.Command{
	Use:   "get <id>",
	Short: "Mostra il dettaglio di un prompt per ID",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		format, _ := cmd.Flags().GetString("format")

		db, err := openVault(vaultPath())
		if err != nil {
			return err
		}
		defer func() { _ = db.Close() }()

		p, err := get(db, args[0])
		if err != nil {
			return err
		}
		out, err := formatPrompt(*p, format)
		if err != nil {
			return err
		}
		_, _ = fmt.Fprint(cmd.OutOrStdout(), out)
		return nil
	},
}

var recentCmd = &cobra.Command{
	Use:   "recent",
	Short: "Lista i prompt usati di recente",
	RunE: func(cmd *cobra.Command, args []string) error {
		limit, _ := cmd.Flags().GetInt("limit")
		format, _ := cmd.Flags().GetString("format")

		db, err := openVault(vaultPath())
		if err != nil {
			return err
		}
		defer func() { _ = db.Close() }()

		prompts, err := recent(db, limit)
		if err != nil {
			return err
		}
		out, err := formatPrompts(prompts, format)
		if err != nil {
			return err
		}
		_, _ = fmt.Fprint(cmd.OutOrStdout(), out)
		return nil
	},
}

var renderCmd = &cobra.Command{
	Use:   "render <id>",
	Short: "Compila un prompt sostituendo i segnaposti {{...}} con i valori forniti",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		varKVs, _ := cmd.Flags().GetStringSlice("var")
		varFile, _ := cmd.Flags().GetString("var-file")

		vars := map[string]string{}
		if varFile != "" {
			b, err := os.ReadFile(varFile)
			if err != nil {
				return fmt.Errorf("lettura var-file: %w", err)
			}
			if err := yaml.Unmarshal(b, &vars); err != nil {
				return fmt.Errorf("parsing var-file: %w", err)
			}
		}
		for _, kv := range varKVs {
			parts := strings.SplitN(kv, "=", 2)
			if len(parts) != 2 {
				return fmt.Errorf("--var %q non valida (atteso key=value)", kv)
			}
			vars[parts[0]] = parts[1]
		}

		db, err := openVault(vaultPath())
		if err != nil {
			return err
		}
		defer func() { _ = db.Close() }()

		p, err := get(db, args[0])
		if err != nil {
			return err
		}

		compilato := compila(p.Body, vars)
		_, _ = fmt.Fprint(cmd.OutOrStdout(), compilato)
		if !strings.HasSuffix(compilato, "\n") {
			_, _ = fmt.Fprintln(cmd.OutOrStdout())
		}
		// Avviso su stderr per segnaposti non compilati (utile in pipe).
		nonCompilati := []string{}
		for _, s := range estraiSegnaposti(p.Body) {
			if v, ok := vars[s]; !ok || strings.TrimSpace(v) == "" {
				nonCompilati = append(nonCompilati, s)
			}
		}
		if len(nonCompilati) > 0 {
			_, _ = fmt.Fprintf(cmd.ErrOrStderr(),
				"[pap] segnaposti non compilati: %s\n", strings.Join(nonCompilati, ", "))
		}
		return nil
	},
}

func init() {
	searchCmd.Flags().IntP("limit", "n", 10, "Numero massimo di risultati")
	searchCmd.Flags().String("target", "", "Filtra per modello target (es. claude-sonnet)")
	searchCmd.Flags().String("tag", "", "Filtra per tag (match esatto)")
	searchCmd.Flags().String("format", "table", "Formato output: table|json|yaml|plain")

	getCmd.Flags().String("format", "table", "Formato output: table|json|yaml|plain")

	recentCmd.Flags().IntP("limit", "n", 10, "Numero massimo di risultati")
	recentCmd.Flags().String("format", "table", "Formato output: table|json|yaml|plain")

	renderCmd.Flags().StringSlice("var", nil, "Coppia key=value per i segnaposti (ripetibile)")
	renderCmd.Flags().String("var-file", "", "File YAML con mappa key:value per i segnaposti")

	rootCmd.AddCommand(versionCmd, searchCmd, getCmd, recentCmd, renderCmd)
	// Cobra aggiunge automaticamente il sub-comando `completion` per bash/zsh/fish/powershell.
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
