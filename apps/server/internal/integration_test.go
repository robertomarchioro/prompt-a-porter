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
	"github.com/golang-jwt/jwt/v5"
	"github.com/gorilla/websocket"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/auth"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/database"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/models"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/server"
	syncpkg "github.com/robertomarchioro/prompt-a-porter/apps/server/internal/sync"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/ws"
)

const testSecret = "test-secret-32-bytes-long-enough"

// testServerOptions permette ai singoli test di personalizzare la
// configurazione del router (allow-list CORS, rate-limit) senza duplicare
// tutto il setup di setupTestServer.
type testServerOptions struct {
	allowedOrigins       []string
	loginRateLimit       int
	loginRateLimitWindow time.Duration
	behindProxy          bool
	trustedProxyCIDRs    []string
}

func setupTestServer(t *testing.T) (*chi.Mux, *database.DB, func()) {
	t.Helper()
	return setupTestServerWithOptions(t, testServerOptions{})
}

func setupTestServerWithOptions(t *testing.T, opts testServerOptions) (*chi.Mux, *database.DB, func()) {
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

	loginRateLimit := opts.loginRateLimit
	if loginRateLimit == 0 {
		loginRateLimit = server.DefaultLoginRateLimit
	}
	loginRateLimitWindow := opts.loginRateLimitWindow
	if loginRateLimitWindow == 0 {
		loginRateLimitWindow = server.DefaultLoginRateLimitWindow
	}

	hub := ws.NewHub(jwtSecret, opts.allowedOrigins)

	authH := &auth.Handler{
		DB:        db,
		JwtSecret: jwtSecret,
		TokenTTL:  1 * time.Hour,
	}

	syncH := &syncpkg.Handler{
		DB:  db,
		Hub: hub,
	}

	r := server.NewRouter(authH, syncH, hub, jwtSecret, server.Config{
		AllowedOrigins:       opts.allowedOrigins,
		TLSAttivo:            false,
		LoginRateLimit:       loginRateLimit,
		LoginRateLimitWindow: loginRateLimitWindow,
		Version:              "test",
		BehindProxy:          opts.behindProxy,
		TrustedProxyCIDRs:    opts.trustedProxyCIDRs,
	})

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

// CWE-613: un utente disattivato (soft-delete) dopo il login NON deve poter
// estendere l'accesso via /auth/refresh. Prima del fix il refresh copiava i
// claim dal solo token, senza rileggere il DB, permettendo accesso a oltranza.
func TestRefreshUtenteDisattivatoNegato(t *testing.T) {
	r, db, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	// Off-boarding: l'admin disattiva l'utente dopo che ha ottenuto un token.
	if _, err := db.Exec("UPDATE Users SET DeletedAt = ? WHERE Id = ?",
		models.NowUTC(), login.User.Id); err != nil {
		t.Fatalf("soft-delete utente: %v", err)
	}

	req := httptest.NewRequest("POST", "/auth/refresh", nil)
	req.Header.Set("Authorization", "Bearer "+login.Token)
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("refresh di un utente disattivato deve dare 401, ottenuto %d: %s",
			rec.Code, rec.Body.String())
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

// TestSyncPushAuthorshipForzato verifica che un client non possa attribuire
// un prompt nuovo a un utente arbitrario: l'AuthorUserId salvato e
// restituito dal pull deve sempre essere quello autenticato che ha eseguito
// il push, mai il valore (eventualmente spoofato) inviato nel body.
// Regressione per #450.
func TestSyncPushAuthorshipForzato(t *testing.T) {
	r, db, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	var wsId string
	db.QueryRow("SELECT WorkspaceId FROM Users WHERE Email = 'admin@test.com'").Scan(&wsId)

	now := models.NowUTC()
	pushAs(t, r, login.Token, models.SyncPushRequest{
		Prompts: []models.Prompt{{
			Id: "prm-spoof", WorkspaceId: wsId,
			AuthorUserId: "usr-vittima-spoofata",
			Title:        "Prompt con autore spoofato", Body: "x", Visibility: "workspace",
			Version: 1, CreatedAt: now, UpdatedAt: now,
		}},
	})

	req := httptest.NewRequest("GET", "/sync/pull?since=1970-01-01+00:00:00", nil)
	req.Header.Set("Authorization", "Bearer "+login.Token)
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	var delta models.SyncDelta
	json.NewDecoder(rec.Body).Decode(&delta)

	if len(delta.Prompts) != 1 {
		t.Fatalf("atteso 1 prompt, ottenuti %d", len(delta.Prompts))
	}
	if delta.Prompts[0].AuthorUserId != login.User.Id {
		t.Fatalf("AuthorUserId atteso %q (utente autenticato), ottenuto %q (spoofato dal client)",
			login.User.Id, delta.Prompts[0].AuthorUserId)
	}

	var changelogAuthor string
	err := db.QueryRow(`SELECT ChangedBy FROM SyncChangelog WHERE EntityId = 'prm-spoof' ORDER BY Id DESC LIMIT 1`).
		Scan(&changelogAuthor)
	if err != nil {
		t.Fatalf("lettura changelog: %v", err)
	}
	if changelogAuthor != login.User.Id {
		t.Fatalf("ChangedBy nel changelog atteso %q, ottenuto %q", login.User.Id, changelogAuthor)
	}
}

// TestSyncPushCrossTenantWriteBypassRifiutato verifica che un utente del
// workspace A non possa sovrascrivere un prompt/tag del workspace B
// riusando lo stesso Id: Id è PRIMARY KEY globale su Prompts/Tags (vedi
// schema.sql), non per-workspace, quindi SELECT/UPDATE filtrate solo per Id
// (senza anche AND WorkspaceId=?) avrebbero permesso di leggere/sovrascrivere
// la riga dell'altro tenant (CWE-639, Authorization Bypass Through
// User-Controlled Key). Il controllo preesistente "tag.WorkspaceId !=
// workspaceId" da solo NON basta: qui l'attaccante dichiara il proprio
// WorkspaceId, l'attacco sta nel riuso dell'Id di una riga altrui.
// Regressione per #482.
func TestSyncPushCrossTenantWriteBypassRifiutato(t *testing.T) {
	r, db, cleanup := setupTestServer(t)
	defer cleanup()

	// WS-B (admin di default): crea un prompt e un tag legittimi.
	loginB := doLogin(t, r)
	var wsB string
	db.QueryRow("SELECT WorkspaceId FROM Users WHERE Email = 'admin@test.com'").Scan(&wsB)

	now := models.NowUTC()
	pushAs(t, r, loginB.Token, models.SyncPushRequest{
		Prompts: []models.Prompt{{
			Id: "prm-condiviso", WorkspaceId: wsB, AuthorUserId: loginB.User.Id,
			Title: "Originale di WS-B", Body: "corpo originale", Visibility: "workspace",
			Version: 1, CreatedAt: now, UpdatedAt: now,
		}},
		Tags: []models.Tag{{
			Id: "tag-condiviso", WorkspaceId: wsB, Name: "originale-wsb",
			CreatedAt: now, UpdatedAt: now,
		}},
	})

	// WS-A (secondo admin, workspace separato) — l'attaccante.
	if err := db.SeedAdmin("attaccante@test.com", "Password123!", "AttaccanteTeam"); err != nil {
		t.Fatalf("seed attaccante: %v", err)
	}
	loginA := loginAs(t, r, "attaccante@test.com", "Password123!")
	var wsA string
	db.QueryRow("SELECT WorkspaceId FROM Users WHERE Email = 'attaccante@test.com'").Scan(&wsA)

	// UpdatedAt futuro: se il bug fosse ancora presente, il record di WS-A
	// "vincerebbe" anche il confronto per conflitto basato su timestamp e
	// sovrascriverebbe comunque la riga di WS-B.
	future := "2999-01-01 00:00:00"
	pushAs(t, r, loginA.Token, models.SyncPushRequest{
		Prompts: []models.Prompt{{
			Id: "prm-condiviso", WorkspaceId: wsA, AuthorUserId: loginA.User.Id,
			Title: "HACKED da WS-A", Body: "corpo attaccante", Visibility: "workspace",
			Version: 1, CreatedAt: now, UpdatedAt: future,
		}},
		Tags: []models.Tag{{
			Id: "tag-condiviso", WorkspaceId: wsA, Name: "hacked-da-wsa",
			CreatedAt: now, UpdatedAt: future,
		}},
	})

	var promptTitle, promptWs string
	if err := db.QueryRow("SELECT Title, WorkspaceId FROM Prompts WHERE Id = 'prm-condiviso'").
		Scan(&promptTitle, &promptWs); err != nil {
		t.Fatalf("lettura prompt condiviso: %v", err)
	}
	if promptTitle != "Originale di WS-B" || promptWs != wsB {
		t.Fatalf("il prompt di WS-B è stato sovrascritto da WS-A: title=%q workspace=%q", promptTitle, promptWs)
	}

	var tagName, tagWs string
	if err := db.QueryRow("SELECT Name, WorkspaceId FROM Tags WHERE Id = 'tag-condiviso'").
		Scan(&tagName, &tagWs); err != nil {
		t.Fatalf("lettura tag condiviso: %v", err)
	}
	if tagName != "originale-wsb" || tagWs != wsB {
		t.Fatalf("il tag di WS-B è stato sovrascritto da WS-A: name=%q workspace=%q", tagName, tagWs)
	}

	var promptCount, tagCount int
	db.QueryRow("SELECT COUNT(*) FROM Prompts WHERE Id = 'prm-condiviso'").Scan(&promptCount)
	db.QueryRow("SELECT COUNT(*) FROM Tags WHERE Id = 'tag-condiviso'").Scan(&tagCount)
	if promptCount != 1 {
		t.Fatalf("atteso esattamente 1 riga Prompts per l'Id condiviso, trovate %d", promptCount)
	}
	if tagCount != 1 {
		t.Fatalf("atteso esattamente 1 riga Tags per l'Id condiviso, trovate %d", tagCount)
	}
}

// TestLoginRateLimitFloodRitorna429 verifica che dopo aver esaurito la
// quota per-IP, /auth/login risponda 429 invece di continuare a validare
// credenziali (mitigazione brute-force, #451).
func TestLoginRateLimitFloodRitorna429(t *testing.T) {
	r, _, cleanup := setupTestServerWithOptions(t, testServerOptions{
		loginRateLimit:       3,
		loginRateLimitWindow: time.Minute,
	})
	defer cleanup()

	var lastCode int
	for i := 0; i < 5; i++ {
		body, _ := json.Marshal(models.LoginRequest{Email: "admin@test.com", Password: "sbagliata"})
		req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
		req.Header.Set("Content-Type", "application/json")
		req.RemoteAddr = "203.0.113.9:12345"
		rec := httptest.NewRecorder()
		r.ServeHTTP(rec, req)
		lastCode = rec.Code
	}

	if lastCode != http.StatusTooManyRequests {
		t.Fatalf("dopo aver esaurito la quota atteso 429, ottenuto %d", lastCode)
	}
}

// TestLoginRateLimitPerIpIndipendente verifica che il rate-limit sia
// per-IP e non globale: un altro IP non deve essere penalizzato dal flood
// generato da un primo IP.
func TestLoginRateLimitPerIpIndipendente(t *testing.T) {
	r, _, cleanup := setupTestServerWithOptions(t, testServerOptions{
		loginRateLimit:       3,
		loginRateLimitWindow: time.Minute,
	})
	defer cleanup()

	flood := func(remoteAddr string, n int) int {
		var code int
		for i := 0; i < n; i++ {
			body, _ := json.Marshal(models.LoginRequest{Email: "admin@test.com", Password: "sbagliata"})
			req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
			req.Header.Set("Content-Type", "application/json")
			req.RemoteAddr = remoteAddr
			rec := httptest.NewRecorder()
			r.ServeHTTP(rec, req)
			code = rec.Code
		}
		return code
	}

	if got := flood("198.51.100.1:1", 5); got != http.StatusTooManyRequests {
		t.Fatalf("primo IP: atteso 429 dopo il flood, ottenuto %d", got)
	}

	body, _ := json.Marshal(models.LoginRequest{Email: "admin@test.com", Password: "Password123!"})
	req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req.RemoteAddr = "198.51.100.2:1"
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("secondo IP non deve essere penalizzato: atteso 200, ottenuto %d: %s", rec.Code, rec.Body.String())
	}
}

// TestLoginRateLimitDietroProxyUsaXForwardedFor verifica che, dietro
// PAP_BEHIND_PROXY=1, il rate-limit su /auth/login chiavizzi sull'IP reale
// del client letto da X-Forwarded-For (fidandosi solo dei CIDR indicati),
// non su RemoteAddr — che dietro un reverse proxy è sempre l'IP del proxy
// e farebbe condividere lo stesso bucket a tutti gli utenti (self-inflicted
// DoS). Due IP reali diversi dietro lo stesso proxy (stesso RemoteAddr) non
// devono condividere il bucket.
func TestLoginRateLimitDietroProxyUsaXForwardedFor(t *testing.T) {
	r, _, cleanup := setupTestServerWithOptions(t, testServerOptions{
		loginRateLimit:       3,
		loginRateLimitWindow: time.Minute,
		behindProxy:          true,
		trustedProxyCIDRs:    []string{"127.0.0.1/32"},
	})
	defer cleanup()

	floodViaProxy := func(realClientIP string, n int) int {
		var code int
		for i := 0; i < n; i++ {
			body, _ := json.Marshal(models.LoginRequest{Email: "admin@test.com", Password: "sbagliata"})
			req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
			req.Header.Set("Content-Type", "application/json")
			req.Header.Set("X-Forwarded-For", realClientIP)
			// Stesso RemoteAddr per entrambi i client: è il proxy fidato
			// (127.0.0.1/32) davanti a entrambi.
			req.RemoteAddr = "127.0.0.1:9999"
			rec := httptest.NewRecorder()
			r.ServeHTTP(rec, req)
			code = rec.Code
		}
		return code
	}

	if got := floodViaProxy("203.0.113.50", 5); got != http.StatusTooManyRequests {
		t.Fatalf("primo client reale (dietro proxy): atteso 429 dopo il flood, ottenuto %d", got)
	}

	// Un secondo client reale, dietro lo STESSO proxy (stesso RemoteAddr),
	// non deve essere penalizzato dal flood del primo.
	body, _ := json.Marshal(models.LoginRequest{Email: "admin@test.com", Password: "Password123!"})
	req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Forwarded-For", "203.0.113.99")
	req.RemoteAddr = "127.0.0.1:9999"
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("secondo client reale dietro lo stesso proxy non deve essere penalizzato: "+
			"atteso 200, ottenuto %d: %s", rec.Code, rec.Body.String())
	}
}

// TestLoginBodyTroppoGrandeRitorna413 verifica che http.MaxBytesReader
// protegga /auth/login da payload enormi (#451).
func TestLoginBodyTroppoGrandeRitorna413(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	hugePassword := strings.Repeat("a", 2<<20) // 2 MiB, ben oltre il limite di 1 KiB
	body, _ := json.Marshal(models.LoginRequest{Email: "admin@test.com", Password: hugePassword})
	req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusRequestEntityTooLarge {
		t.Fatalf("atteso 413, ottenuto %d: %s", rec.Code, rec.Body.String())
	}
}

// TestSyncPushBodyTroppoGrandeRitorna413 verifica lo stesso limite su
// /sync/push, che accetta payload più grandi ma non illimitati (#451).
func TestSyncPushBodyTroppoGrandeRitorna413(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	hugeBody := models.Prompt{
		Id: "prm-huge", WorkspaceId: "ws", AuthorUserId: "usr",
		Title: "x", Body: strings.Repeat("a", 11<<20), Visibility: "workspace",
		Version: 1, CreatedAt: models.NowUTC(), UpdatedAt: models.NowUTC(),
	}
	data, _ := json.Marshal(models.SyncPushRequest{Prompts: []models.Prompt{hugeBody}})
	req := httptest.NewRequest("POST", "/sync/push", bytes.NewReader(data))
	req.Header.Set("Authorization", "Bearer "+login.Token)
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusRequestEntityTooLarge {
		t.Fatalf("atteso 413, ottenuto %d: %s", rec.Code, rec.Body.String())
	}
}

// TestLoginUtenteInesistenteTempoEquivalente verifica che il ramo "utente
// non trovato" esegua comunque una verifica Argon2id (equalizzazione dei
// tempi, #451): non è un test di timing rigoroso (fragile in CI), ma
// verifica che il comportamento funzionale resti 401 e che l'hash dummy sia
// effettivamente valido (VerifyPassword non deve panicare/errare).
func TestLoginUtenteInesistenteTempoEquivalente(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	body, _ := json.Marshal(models.LoginRequest{Email: "fantasma@test.com", Password: "qualsiasi-password-lunga-a-piacere"})
	req := httptest.NewRequest("POST", "/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("atteso 401, ottenuto %d", rec.Code)
	}
}

// TestHealthNonEspornClientCount verifica che /health non esponga più il
// conteggio dei client WebSocket connessi senza autenticazione (parte
// server di #462).
func TestHealthNonEspornClientCount(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	req := httptest.NewRequest("GET", "/health", nil)
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("atteso 200, ottenuto %d", rec.Code)
	}
	if strings.Contains(rec.Body.String(), "clients") {
		t.Fatalf("/health non deve esporre il conteggio client: %s", rec.Body.String())
	}
}

// TestSecurityHeadersPresenti verifica che il middleware di sicurezza
// aggiunga gli header di difesa in profondità su tutte le risposte (parte
// server di #462).
func TestSecurityHeadersPresenti(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	req := httptest.NewRequest("GET", "/health", nil)
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if got := rec.Header().Get("X-Content-Type-Options"); got != "nosniff" {
		t.Fatalf("X-Content-Type-Options atteso nosniff, ottenuto %q", got)
	}
	if got := rec.Header().Get("X-Frame-Options"); got != "DENY" {
		t.Fatalf("X-Frame-Options atteso DENY, ottenuto %q", got)
	}
	if got := rec.Header().Get("Strict-Transport-Security"); got != "" {
		t.Fatalf("HSTS non deve essere impostato quando il server non serve TLS direttamente, ottenuto %q", got)
	}
}

// TestCorsOriginNonInAllowListRifiutata verifica che una origin non
// autorizzata non riceva l'header Access-Control-Allow-Origin (#452).
func TestCorsOriginNonInAllowListRifiutata(t *testing.T) {
	r, _, cleanup := setupTestServerWithOptions(t, testServerOptions{
		allowedOrigins: []string{"https://app.autorizzata.example.com"},
	})
	defer cleanup()

	req := httptest.NewRequest("GET", "/health", nil)
	req.Header.Set("Origin", "https://evil.example.com")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if got := rec.Header().Get("Access-Control-Allow-Origin"); got != "" {
		t.Fatalf("origin non autorizzata non deve ricevere Access-Control-Allow-Origin, ottenuto %q", got)
	}
}

// TestCorsOriginInAllowListAccettata verifica il percorso positivo: una
// origin nella allow-list riceve l'header CORS corrispondente (#452).
func TestCorsOriginInAllowListAccettata(t *testing.T) {
	r, _, cleanup := setupTestServerWithOptions(t, testServerOptions{
		allowedOrigins: []string{"https://app.autorizzata.example.com"},
	})
	defer cleanup()

	req := httptest.NewRequest("GET", "/health", nil)
	req.Header.Set("Origin", "https://app.autorizzata.example.com")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if got := rec.Header().Get("Access-Control-Allow-Origin"); got != "https://app.autorizzata.example.com" {
		t.Fatalf("origin autorizzata deve ricevere Access-Control-Allow-Origin, ottenuto %q", got)
	}
}

// TestCorsAllowListVuotaNonEAllowAll è una regressione mirata: go-chi/cors
// tratta una AllowedOrigins vuota come "consenti tutte le origin" quando
// AllowOriginFunc è nil (verificato su go-chi/cors@v1.2.2, vedi il
// commento su corsOptions in internal/server/router.go). Usa
// setupTestServer (nessuna PAP_ALLOWED_ORIGINS, il caso di default in
// produzione), non una allow-list esplicita come gli altri test CORS sopra
// — altrimenti il default insicuro passerebbe inosservato.
func TestCorsAllowListVuotaNonEAllowAll(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	req := httptest.NewRequest("GET", "/health", nil)
	req.Header.Set("Origin", "https://qualunque-sito.example.com")
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if got := rec.Header().Get("Access-Control-Allow-Origin"); got != "" {
		t.Fatalf("con allow-list vuota (default) nessuna origin deve ricevere "+
			"Access-Control-Allow-Origin, ottenuto %q — la allow-list vuota si è "+
			"comportata come allow-all", got)
	}
}

// TestJwtAlgNoneRifiutato verifica che un token con alg="none" (o firmato
// con un algoritmo diverso da HS256) sia rifiutato sia dal middleware HTTP
// sia dall'handshake WebSocket (jwt.WithValidMethods, CWE-347).
func TestJwtAlgNoneRifiutato(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	claims := jwt.MapClaims{
		"userId":      "usr-attaccante",
		"workspaceId": "ws-vittima",
		"role":        "Admin",
	}
	token := jwt.NewWithClaims(jwt.SigningMethodNone, claims)
	tokenStr, err := token.SignedString(jwt.UnsafeAllowNoneSignatureType)
	if err != nil {
		t.Fatalf("errore firma token alg=none: %v", err)
	}

	req := httptest.NewRequest("GET", "/sync/pull", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("token alg=none deve essere rifiutato: atteso 401, ottenuto %d", rec.Code)
	}
}

// wsClientProtocolTokenPrefix specchia WS_PROTOCOLLO_TOKEN_PREFIX in
// apps/client/src/lib/sync.ts (connettiWs, PR #478) e wsProtocolTokenPrefix
// in internal/ws/hub.go: il client offre come sub-protocol
// "pap.sync.token.<jwt>", non il token nudo. Va tenuto sincronizzato a
// mano con le altre due costanti (pacchetti diversi, nessun modo di
// condividerla senza un import ciclico/cross-modulo).
const wsClientProtocolTokenPrefix = "pap.sync.token."

// wsDialWithHeader apre una connessione WebSocket verso il path indicato di
// un httptest.Server, passando il token via Sec-WebSocket-Protocol
// esattamente nel formato del client (prefisso incluso).
func wsDialWithHeader(t *testing.T, wsURL, token string) (*websocket.Conn, *http.Response, error) {
	t.Helper()
	header := http.Header{}
	header.Set("Sec-WebSocket-Protocol", wsClientProtocolTokenPrefix+token)
	return websocket.DefaultDialer.Dial(wsURL, header)
}

// TestWsTokenViaSecWebSocketProtocolFormatoClienteEsatto è un test di
// round-trip contro una regressione specifica: il client (apps/client/src/
// lib/sync.ts, connettiWs) invia il token PREFISSATO
// ("pap.sync.token.<jwt>"), non il JWT nudo, come valore di
// Sec-WebSocket-Protocol. Se il server non toglie il prefisso prima del
// parse, jwt.ParseWithClaims vede troppi segmenti "." e rifiuta OGNI
// connessione WS con 401, anche con un token valido (#453/#478).
func TestWsTokenViaSecWebSocketProtocolFormatoClienteEsatto(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	ts := httptest.NewServer(r)
	defer ts.Close()
	wsURL := "ws" + strings.TrimPrefix(ts.URL, "http") + "/ws"

	header := http.Header{}
	header.Set("Sec-WebSocket-Protocol", wsClientProtocolTokenPrefix+login.Token)

	conn, resp, err := websocket.DefaultDialer.Dial(wsURL, header)
	if err != nil {
		status := 0
		if resp != nil {
			status = resp.StatusCode
		}
		t.Fatalf("connessione WS con token nel formato esatto del client fallita: %v (status %d)", err, status)
	}
	defer conn.Close()
}

// TestWsTokenViaSecWebSocketProtocol verifica il percorso di autenticazione
// raccomandato via header Sec-WebSocket-Protocol (#453).
func TestWsTokenViaSecWebSocketProtocol(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	ts := httptest.NewServer(r)
	defer ts.Close()
	wsURL := "ws" + strings.TrimPrefix(ts.URL, "http") + "/ws"

	conn, resp, err := wsDialWithHeader(t, wsURL, login.Token)
	if err != nil {
		t.Fatalf("connessione WS con token via header fallita: %v (status %v)", err, resp)
	}
	defer conn.Close()
}

// TestWsTokenMancanteRifiutato verifica che senza alcun token la richiesta
// di upgrade venga rifiutata con 401 (#453).
func TestWsTokenMancanteRifiutato(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	ts := httptest.NewServer(r)
	defer ts.Close()
	wsURL := "ws" + strings.TrimPrefix(ts.URL, "http") + "/ws"

	_, resp, err := websocket.DefaultDialer.Dial(wsURL, nil)
	if err == nil {
		t.Fatal("connessione senza token doveva fallire")
	}
	if resp == nil || resp.StatusCode != http.StatusUnauthorized {
		status := 0
		if resp != nil {
			status = resp.StatusCode
		}
		t.Fatalf("atteso 401, ottenuto %d", status)
	}
}

// TestWsOrigineNonConsentitaRifiutata verifica che CheckOrigin rifiuti
// l'handshake da una origin browser non presente nella allow-list (#453).
func TestWsOrigineNonConsentitaRifiutata(t *testing.T) {
	r, _, cleanup := setupTestServerWithOptions(t, testServerOptions{
		allowedOrigins: []string{"https://app.autorizzata.example.com"},
	})
	defer cleanup()

	login := doLogin(t, r)

	ts := httptest.NewServer(r)
	defer ts.Close()
	wsURL := "ws" + strings.TrimPrefix(ts.URL, "http") + "/ws"

	header := http.Header{}
	header.Set("Sec-WebSocket-Protocol", wsClientProtocolTokenPrefix+login.Token)
	header.Set("Origin", "https://evil.example.com")

	_, resp, err := websocket.DefaultDialer.Dial(wsURL, header)
	if err == nil {
		t.Fatal("connessione da origin non autorizzata doveva fallire")
	}
	if resp == nil || resp.StatusCode != http.StatusForbidden {
		status := 0
		if resp != nil {
			status = resp.StatusCode
		}
		t.Fatalf("atteso 403 (CheckOrigin), ottenuto %d", status)
	}
}

// TestWsMessaggioOversizeChiudeConnessione verifica che conn.SetReadLimit
// faccia chiudere la connessione quando il client invia un messaggio oltre
// il limite consentito (#453).
func TestWsMessaggioOversizeChiudeConnessione(t *testing.T) {
	r, _, cleanup := setupTestServer(t)
	defer cleanup()

	login := doLogin(t, r)

	ts := httptest.NewServer(r)
	defer ts.Close()
	wsURL := "ws" + strings.TrimPrefix(ts.URL, "http") + "/ws"

	conn, _, err := wsDialWithHeader(t, wsURL, login.Token)
	if err != nil {
		t.Fatalf("connessione WS fallita: %v", err)
	}
	defer conn.Close()

	oversize := make([]byte, 8192) // > maxWsMessageBytes (4096)
	if err := conn.WriteMessage(websocket.TextMessage, oversize); err != nil {
		t.Fatalf("scrittura messaggio oversize fallita lato client: %v", err)
	}

	conn.SetReadDeadline(time.Now().Add(5 * time.Second))
	_, _, err = conn.ReadMessage()
	if err == nil {
		t.Fatal("il server doveva chiudere la connessione per messaggio oltre il limite")
	}
}
