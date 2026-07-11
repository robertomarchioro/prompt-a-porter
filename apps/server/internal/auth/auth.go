package auth

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/golang-jwt/jwt/v5"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/database"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/httpx"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/models"
)

// maxLoginBodyBytes limita il body di /auth/login: email+password non
// superano mai qualche centinaio di byte, 1 KiB lascia ampio margine e
// impedisce a un client di inviare payload enormi (CWE-400).
const maxLoginBodyBytes = 1 << 10 // 1 KiB

type Handler struct {
	DB        *database.DB
	JwtSecret []byte
	TokenTTL  time.Duration
}

// dummyPasswordHash è un hash argon2id valido, con gli stessi parametri di
// costo usati da HashPassword, calcolato una sola volta all'avvio. Viene
// verificato (scartando l'esito) quando l'email non esiste, per far durare
// il ramo "utente non trovato" quanto il ramo "password errata": altrimenti
// il primo ritornerebbe quasi istantaneamente (nessun Argon2id eseguito) e
// un attaccante potrebbe enumerare le email registrate misurando i tempi di
// risposta del login (CWE-208, User Enumeration via Timing Attack).
var dummyPasswordHash = mustDummyPasswordHash()

func mustDummyPasswordHash() string {
	hash, err := database.HashPassword("pap-timing-equalization-dummy")
	if err != nil {
		// In pratica non accade mai (rand.Read fallisce solo per errori di
		// sistema gravi): niente panic per non impedire l'avvio, la
		// verifica userà comunque un formato valido.
		log.Printf("ATTENZIONE: impossibile generare l'hash dummy per l'equalizzazione dei tempi di login: %v", err)
		return "$argon2id$v=19$m=65536,t=3,p=4$00000000000000000000000000000000$0000000000000000000000000000000000000000000000000000000000000000"
	}
	return hash
}

type Claims struct {
	UserId      string `json:"userId"`
	WorkspaceId string `json:"workspaceId"`
	Role        string `json:"role"`
	jwt.RegisteredClaims
}

func (h *Handler) Login(w http.ResponseWriter, r *http.Request) {
	var req models.LoginRequest
	if !httpx.DecodeJSONLimited(w, r, maxLoginBodyBytes, &req, `{"error":"richiesta non valida"}`) {
		return
	}

	if req.Email == "" || req.Password == "" {
		http.Error(w, `{"error":"email e password obbligatori"}`, http.StatusBadRequest)
		return
	}

	var user models.User
	err := h.DB.QueryRow(`
		SELECT Id, WorkspaceId, Email, DisplayName, Role, PasswordHash, CreatedAt, UpdatedAt
		FROM Users WHERE Email = ? AND DeletedAt IS NULL`,
		req.Email,
	).Scan(&user.Id, &user.WorkspaceId, &user.Email, &user.DisplayName,
		&user.Role, &user.PasswordHash, &user.CreatedAt, &user.UpdatedAt)

	if err != nil {
		// Esegue comunque una verifica Argon2id completa (su un hash dummy)
		// per equalizzare i tempi di risposta col ramo "password errata"
		// sotto — vedi il commento su dummyPasswordHash.
		database.VerifyPassword(req.Password, dummyPasswordHash)
		log.Printf("Login fallito per %s: utente non trovato", req.Email)
		http.Error(w, `{"error":"credenziali non valide"}`, http.StatusUnauthorized)
		return
	}

	if !database.VerifyPassword(req.Password, user.PasswordHash) {
		log.Printf("Login fallito per %s: password errata", req.Email)
		http.Error(w, `{"error":"credenziali non valide"}`, http.StatusUnauthorized)
		return
	}

	expiresAt := time.Now().Add(h.TokenTTL)
	claims := Claims{
		UserId:      user.Id,
		WorkspaceId: user.WorkspaceId,
		Role:        user.Role,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(expiresAt),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			Subject:   user.Id,
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenStr, err := token.SignedString(h.JwtSecret)
	if err != nil {
		log.Printf("Errore firma JWT: %v", err)
		http.Error(w, `{"error":"errore interno"}`, http.StatusInternalServerError)
		return
	}

	log.Printf("Login riuscito: %s", req.Email)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(models.LoginResponse{
		Token:     tokenStr,
		ExpiresAt: expiresAt.Unix(),
		User:      user,
	})
}

func (h *Handler) Refresh(w http.ResponseWriter, r *http.Request) {
	claims, ok := ClaimsFromContext(r.Context())
	if !ok {
		http.Error(w, `{"error":"non autenticato"}`, http.StatusUnauthorized)
		return
	}

	expiresAt := time.Now().Add(h.TokenTTL)
	newClaims := Claims{
		UserId:      claims.UserId,
		WorkspaceId: claims.WorkspaceId,
		Role:        claims.Role,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(expiresAt),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			Subject:   claims.UserId,
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, newClaims)
	tokenStr, err := token.SignedString(h.JwtSecret)
	if err != nil {
		http.Error(w, `{"error":"errore interno"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(models.LoginResponse{
		Token:     tokenStr,
		ExpiresAt: expiresAt.Unix(),
	})
}
