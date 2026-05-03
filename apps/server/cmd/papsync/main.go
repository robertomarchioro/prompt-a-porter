package main

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	chiMiddleware "github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"

	"github.com/anthropics/prompt-a-porter/apps/server/internal/auth"
	"github.com/anthropics/prompt-a-porter/apps/server/internal/database"
	"github.com/anthropics/prompt-a-porter/apps/server/internal/middleware"
	syncHandler "github.com/anthropics/prompt-a-porter/apps/server/internal/sync"
	"github.com/anthropics/prompt-a-porter/apps/server/internal/ws"
)

func main() {
	port := envOr("PAP_PORT", "8443")
	dbPath := envOr("PAP_DB_PATH", "papsync.db")
	adminEmail := os.Getenv("PAP_ADMIN_EMAIL")
	adminPassword := os.Getenv("PAP_ADMIN_PASSWORD")
	wsName := envOr("PAP_WORKSPACE_NAME", "Team")
	jwtSecret := jwtSecretFromEnv()

	db, err := database.Open(dbPath)
	if err != nil {
		log.Fatalf("Errore apertura DB: %v", err)
	}
	defer db.Close()

	if err := db.Migrate(); err != nil {
		log.Fatalf("Errore migrazione: %v", err)
	}

	if adminEmail != "" && adminPassword != "" {
		if err := db.SeedAdmin(adminEmail, adminPassword, wsName); err != nil {
			log.Fatalf("Errore creazione admin: %v", err)
		}
	}

	hub := ws.NewHub(jwtSecret)

	authHandler := &auth.Handler{
		DB:        db,
		JwtSecret: jwtSecret,
		TokenTTL:  24 * time.Hour,
	}

	syncH := &syncHandler.Handler{
		DB:  db,
		Hub: hub,
	}

	r := chi.NewRouter()

	r.Use(chiMiddleware.Recoverer)
	r.Use(middleware.Logger)
	r.Use(cors.Handler(cors.Options{
		AllowedOrigins:   []string{"*"},
		AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders:   []string{"Authorization", "Content-Type"},
		ExposedHeaders:   []string{"Content-Length"},
		AllowCredentials: true,
		MaxAge:           300,
	}))

	r.Get("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		fmt.Fprintf(w, `{"status":"ok","version":"0.1.0","clients":%d}`, hub.ClientCount())
	})

	r.Post("/auth/login", authHandler.Login)

	r.Group(func(r chi.Router) {
		r.Use(middleware.JwtAuth(jwtSecret))

		r.Post("/auth/refresh", authHandler.Refresh)
		r.Get("/sync/pull", syncH.Pull)
		r.Post("/sync/push", syncH.Push)
	})

	r.Get("/ws", hub.HandleWs)

	log.Printf("PaP Sync Server avviato su :%s", port)
	if err := http.ListenAndServe(":"+port, r); err != nil {
		log.Fatalf("Errore server: %v", err)
	}
}

func envOr(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func jwtSecretFromEnv() []byte {
	secret := os.Getenv("PAP_JWT_SECRET")
	if secret != "" {
		return []byte(secret)
	}
	b := make([]byte, 32)
	rand.Read(b)
	generated := hex.EncodeToString(b)
	log.Printf("ATTENZIONE: PAP_JWT_SECRET non impostato, generato casualmente (non persistente tra riavvii)")
	return []byte(generated)
}
