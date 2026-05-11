package auth

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/golang-jwt/jwt/v5"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/database"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/models"
)

type Handler struct {
	DB        *database.DB
	JwtSecret []byte
	TokenTTL  time.Duration
}

type Claims struct {
	UserId      string `json:"userId"`
	WorkspaceId string `json:"workspaceId"`
	Role        string `json:"role"`
	jwt.RegisteredClaims
}

func (h *Handler) Login(w http.ResponseWriter, r *http.Request) {
	var req models.LoginRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"richiesta non valida"}`, http.StatusBadRequest)
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
