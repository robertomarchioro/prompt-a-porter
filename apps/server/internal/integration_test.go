package internal

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/go-chi/chi/v5"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/auth"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/database"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/middleware"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/models"
	syncpkg "github.com/robertomarchioro/prompt-a-porter/apps/server/internal/sync"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/ws"
)

const testSecret = "test-secret-32-bytes-long-enough"

func setupTestServer(t *testing.T) (*chi.Mux, *database.DB, func()) {
	t.Helper()

	dir := t.TempDir()
	dbPath := filepath.Join(dir, "test.db")

	db, err := database.Open(dbPath)
	if err != nil {
		t.Fatalf("apertura DB: %v", err)
	}

	if err := db.Migrate(); err != nil {
		t.Fatalf("migrazione: %v", err)
	}

	if err := db.SeedAdmin("admin@test.com", "Password123!", "TestTeam"); err != nil {
		t.Fatalf("seed admin: %v", err)
	}

	jwtSecret := []byte(testSecret)
	hub := ws.NewHub(jwtSecret)

	authH := &auth.Handler{
		DB:        db,
		JwtSecret: jwtSecret,
		TokenTTL:  1 * time.Hour,
	}

	syncH := &syncpkg.Handler{
		DB:  db,
		Hub: hub,
	}

	r := chi.NewRouter()
	r.Post("/auth/login", authH.Login)

	r.Group(func(r chi.Router) {
		r.Use(middleware.JwtAuth(jwtSecret))
		r.Post("/auth/refresh", authH.Refresh)
		r.Get("/sync/pull", syncH.Pull)
		r.Post("/sync/push", syncH.Push)
	})

	r.Get("/ws", hub.HandleWs)

	cleanup := func() {
		db.Close()
		os.RemoveAll(dir)
	}

	return r, db, cleanup
}

func doLogin(t *testing.T, r *chi.Mux) models.LoginResponse {
	t.Helper()
	body, _ := json.Marshal(models.LoginRequest{Email: "admin@test.com", Password: "Password123!"})
	req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("login atteso 200, ottenuto %d: %s", rec.Code, rec.Body.String())
	}

	var resp models.LoginResponse
	json.NewDecoder(rec.Body).Decode(&resp)
	return resp
}

func loginAs(t *testing.T, r *chi.Mux, email, password string) models.LoginResponse {
	t.Helper()
	body, _ := json.Marshal(models.LoginRequest{Email: email, Password: password})
	req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("login %s atteso 200, ottenuto %d: %s", email, rec.Code, rec.Body.String())
	}
	var resp models.LoginResponse
	json.NewDecoder(rec.Body).Decode(&resp)
	return resp
}

func pushAs(t *testing.T, r *chi.Mux, token string, body models.SyncPushRequest) {
	t.Helper()
	data, _ := json.Marshal(body)
	req := httptest.NewRequest("POST", "/sync/push", bytes.NewReader(data))
	req.Header.Set("Authorization", "Bearer "+token)
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("push atteso 200, ottenuto %d: %s", rec.Code, rec.Body.String())
	}
}

func TestLoginValido(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	resp := doLogin(t, r)

	if resp.Token == "" {
		t.Fatal("token vuoto")
	}
	if resp.User.Email != "admin@test.com" {
		t.Fatalf("email attesa admin@test.com, ottenuta %s", resp.User.Email)
	}
	if resp.User.Role != "Admin" {
		t.Fatalf("ruolo atteso Admin, ottenuto %s", resp.User.Role)
	}
}

func TestLoginPasswordErrata(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	body, _ := json.Marshal(models.LoginRequest{Email: "admin@test.com", Password: "sbagliata"})
	req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("atteso 401, ottenuto %d", rec.Code)
	}
}

func TestLoginUtenteInesistente(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	body, _ := json.Marshal(models.LoginRequest{Email: "nessuno@test.com", Password: "qualsiasi"})
	req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("atteso 401, ottenuto %d", rec.Code)
	}
}

func TestRefreshToken(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	req := httptest.NewRequest("POST", "/auth/refresh", nil)
	req.Header.Set("Authorization", "Bearer "+login.Token)
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("atteso 200, ottenuto %d", rec.Code)
	}

	var resp models.LoginResponse
	json.NewDecoder(rec.Body).Decode(&resp)
	if resp.Token == "" {
		t.Fatal("nuovo token vuoto")
	}
}

func TestSyncPullVuoto(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	req := httptest.NewRequest("GET", "/sync/pull?since=1970-01-01+00:00:00", nil)
	req.Header.Set("Authorization", "Bearer "+login.Token)
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("atteso 200, ottenuto %d: %s", rec.Code, rec.Body.String())
	}

	var delta models.SyncDelta
	json.NewDecoder(rec.Body).Decode(&delta)

	if len(delta.Prompts) != 0 {
		t.Fatalf("attesi 0 prompt, ottenuti %d", len(delta.Prompts))
	}
	if delta.Timestamp == "" {
		t.Fatal("timestamp vuoto")
	}
}

func TestSyncPushEPull(t *testing.T) {
	r, db, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	var wsId string
	db.QueryRow("SELECT WorkspaceId FROM Users WHERE Email = 'admin@test.com'").Scan(&wsId)

	pushBody := models.SyncPushRequest{
		Prompts: []models.Prompt{
			{
				Id:           "prm-test001",
				WorkspaceId:  wsId,
				AuthorUserId: login.User.Id,
				Title:        "Prompt Test Sync",
				Body:         "Corpo con {{segnaposto}}",
				Visibility:   "workspace",
				Version:      1,
				CreatedAt:    models.NowUTC(),
				UpdatedAt:    models.NowUTC(),
			},
		},
		Tags: []models.Tag{
			{
				Id:          "tag-test001",
				WorkspaceId: wsId,
				Name:        "sync-test",
				CreatedAt:   models.NowUTC(),
				UpdatedAt:   models.NowUTC(),
			},
		},
		PromptTags: []models.PromptTag{
			{PromptId: "prm-test001", TagId: "tag-test001"},
		},
	}

	data, _ := json.Marshal(pushBody)
	req := httptest.NewRequest("POST", "/sync/push", bytes.NewReader(data))
	req.Header.Set("Authorization", "Bearer "+login.Token)
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("push atteso 200, ottenuto %d: %s", rec.Code, rec.Body.String())
	}

	var pushResp models.SyncPushResponse
	json.NewDecoder(rec.Body).Decode(&pushResp)

	if pushResp.Accepted != 2 {
		t.Fatalf("attesi 2 accepted, ottenuti %d", pushResp.Accepted)
	}

	req2 := httptest.NewRequest("GET", "/sync/pull?since=1970-01-01+00:00:00", nil)
	req2.Header.Set("Authorization", "Bearer "+login.Token)
	rec2 := httptest.NewRecorder()
	r.ServeHTTP(rec2, req2)

	if rec2.Code != http.StatusOK {
		t.Fatalf("pull atteso 200, ottenuto %d", rec2.Code)
	}

	var delta models.SyncDelta
	json.NewDecoder(rec2.Body).Decode(&delta)

	if len(delta.Prompts) != 1 {
		t.Fatalf("atteso 1 prompt, ottenuti %d", len(delta.Prompts))
	}
	if delta.Prompts[0].Title != "Prompt Test Sync" {
		t.Fatalf("titolo atteso 'Prompt Test Sync', ottenuto '%s'", delta.Prompts[0].Title)
	}
	if len(delta.Tags) != 1 {
		t.Fatalf("atteso 1 tag, ottenuti %d", len(delta.Tags))
	}
	if len(delta.PromptTags) != 1 {
		t.Fatalf("attesa 1 associazione, ottenute %d", len(delta.PromptTags))
	}
}

// TestSyncPushPromptTagCrossWorkspaceRifiutato verifica che un client
// autenticato in un workspace NON possa creare associazioni PromptTag che
// puntano a prompt/tag di un altro workspace (CWE-639). Regressione per il
// fix in sync/handler.go pushDelta.
func TestSyncPushPromptTagCrossWorkspaceRifiutato(t *testing.T) {
	r, db, cleanup := setupTestServer(t)
	defer cleanup()

	// WS1 (admin di default): inserisce un prompt e un tag legittimi.
	login1 := doLogin(t, r)
	var ws1 string
	db.QueryRow("SELECT WorkspaceId FROM Users WHERE Email = 'admin@test.com'").Scan(&ws1)

	now := models.NowUTC()
	pushAs(t, r, login1.Token, models.SyncPushRequest{
		Prompts: []models.Prompt{{
			Id: "prm-ws1", WorkspaceId: ws1, AuthorUserId: login1.User.Id,
			Title: "WS1 prompt", Body: "x", Visibility: "workspace",
			Version: 1, CreatedAt: now, UpdatedAt: now,
		}},
		Tags: []models.Tag{{
			Id: "tag-ws1", WorkspaceId: ws1, Name: "ws1tag",
			CreatedAt: now, UpdatedAt: now,
		}},
	})

	// WS2 (secondo admin in workspace separato).
	if err := db.SeedAdmin("evil@test.com", "Password123!", "EvilTeam"); err != nil {
		t.Fatalf("seed evil: %v", err)
	}
	login2 := loginAs(t, r, "evil@test.com", "Password123!")

	// WS2 tenta di collegare prompt+tag di WS1: deve essere ignorato.
	pushAs(t, r, login2.Token, models.SyncPushRequest{
		PromptTags: []models.PromptTag{{PromptId: "prm-ws1", TagId: "tag-ws1"}},
	})

	var count int
	db.QueryRow("SELECT COUNT(*) FROM PromptTags WHERE PromptId = 'prm-ws1' AND TagId = 'tag-ws1'").
		Scan(&count)
	if count != 0 {
		t.Fatalf("PromptTag cross-workspace non deve essere inserito, trovate %d righe", count)
	}

	// Sanita': WS1 puo' ancora associare i propri prompt/tag.
	pushAs(t, r, login1.Token, models.SyncPushRequest{
		PromptTags: []models.PromptTag{{PromptId: "prm-ws1", TagId: "tag-ws1"}},
	})
	db.QueryRow("SELECT COUNT(*) FROM PromptTags WHERE PromptId = 'prm-ws1' AND TagId = 'tag-ws1'").
		Scan(&count)
	if count != 1 {
		t.Fatalf("PromptTag legittimo stesso-workspace deve essere inserito, trovate %d righe", count)
	}
}

func TestSyncConflict(t *testing.T) {
	r, db, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	var wsId string
	db.QueryRow("SELECT WorkspaceId FROM Users WHERE Email = 'admin@test.com'").Scan(&wsId)

	now := models.NowUTC()
	prompt := models.Prompt{
		Id: "prm-conflict", WorkspaceId: wsId, AuthorUserId: login.User.Id,
		Title: "Originale", Body: "corpo", Visibility: "workspace",
		Version: 1, CreatedAt: now, UpdatedAt: now,
	}

	data, _ := json.Marshal(models.SyncPushRequest{Prompts: []models.Prompt{prompt}})
	req := httptest.NewRequest("POST", "/sync/push", bytes.NewReader(data))
	req.Header.Set("Authorization", "Bearer "+login.Token)
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("primo push atteso 200, ottenuto %d", rec.Code)
	}

	prompt.Title = "Conflitto"
	prompt.UpdatedAt = "2020-01-01 00:00:00"
	data2, _ := json.Marshal(models.SyncPushRequest{Prompts: []models.Prompt{prompt}})
	req2 := httptest.NewRequest("POST", "/sync/push", bytes.NewReader(data2))
	req2.Header.Set("Authorization", "Bearer "+login.Token)
	req2.Header.Set("Content-Type", "application/json")
	rec2 := httptest.NewRecorder()
	r.ServeHTTP(rec2, req2)

	var resp models.SyncPushResponse
	json.NewDecoder(rec2.Body).Decode(&resp)

	if resp.Conflicts != 1 {
		t.Fatalf("atteso 1 conflitto, ottenuti %d", resp.Conflicts)
	}
}

func TestEndpointSenzaAuth(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	endpoints := []struct {
		method string
		path   string
	}{
		{"GET", "/sync/pull"},
		{"POST", "/sync/push"},
		{"POST", "/auth/refresh"},
	}

	for _, ep := range endpoints {
		req := httptest.NewRequest(ep.method, ep.path, nil)
		rec := httptest.NewRecorder()
		r.ServeHTTP(rec, req)

		if rec.Code != http.StatusUnauthorized {
			t.Errorf("%s %s: atteso 401, ottenuto %d", ep.method, ep.path, rec.Code)
		}
	}
}

func TestTokenInvalido(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	req := httptest.NewRequest("GET", "/sync/pull", nil)
	req.Header.Set("Authorization", "Bearer token-totalmente-invalido")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("atteso 401, ottenuto %d", rec.Code)
	}
}

func TestHashVerifyPassword(t *testing.T) {
	password := "TestPassword123!"
	hash, err := database.HashPassword(password)
	if err != nil {
		t.Fatalf("errore hash: %v", err)
	}

	if !strings.HasPrefix(hash, "$argon2id$") {
		t.Fatalf("hash non inizia con $argon2id$: %s", hash)
	}

	if !database.VerifyPassword(password, hash) {
		t.Fatal("verifica password corretta fallita")
	}

	if database.VerifyPassword("sbagliata", hash) {
		t.Fatal("verifica password errata accettata")
	}
}

func TestGeneraId(t *testing.T) {
	id1 := database.GeneraId("prm")
	id2 := database.GeneraId("prm")

	if id1 == id2 {
		t.Fatal("due ID generati sono uguali")
	}
	if !strings.HasPrefix(id1, "prm-") {
		t.Fatalf("id non inizia con prefisso: %s", id1)
	}
}

func TestMigrazioneIdempotente(t *testing.T) {
	dir := t.TempDir()
	dbPath := filepath.Join(dir, "test.db")

	db, err := database.Open(dbPath)
	if err != nil {
		t.Fatalf("apertura DB: %v", err)
	}
	defer db.Close()

	if err := db.Migrate(); err != nil {
		t.Fatalf("prima migrazione: %v", err)
	}
	if err := db.Migrate(); err != nil {
		t.Fatalf("seconda migrazione: %v", err)
	}

	var count int
	db.QueryRow("SELECT COUNT(*) FROM _Migrazioni").Scan(&count)
	if count != 1 {
		t.Fatalf("attesa 1 migrazione, trovate %d", count)
	}
}

func TestSeedAdminIdempotente(t *testing.T) {
	dir := t.TempDir()
	dbPath := filepath.Join(dir, "test.db")

	db, err := database.Open(dbPath)
	if err != nil {
		t.Fatalf("apertura DB: %v", err)
	}
	defer db.Close()
	db.Migrate()

	db.SeedAdmin("admin@test.com", "pass1", "Team1")
	db.SeedAdmin("admin@test.com", "pass2", "Team2")

	var count int
	db.QueryRow("SELECT COUNT(*) FROM Users WHERE Email = 'admin@test.com'").Scan(&count)
	if count != 1 {
		t.Fatalf("atteso 1 admin, trovati %d", count)
	}
}
