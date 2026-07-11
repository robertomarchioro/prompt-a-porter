package main

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/auth"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/config"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/database"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/server"
	syncHandler "github.com/robertomarchioro/prompt-a-porter/apps/server/internal/sync"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/ws"
)

// minJwtSecretBytes è la lunghezza minima accettata per PAP_JWT_SECRET: un
// segreto HMAC più corto è vulnerabile a brute-force offline e rende i JWT
// falsificabili (CWE-326, Inadequate Encryption Strength).
const minJwtSecretBytes = 32

func main() {
	port := envOr("PAP_PORT", "8443")
	dbPath := envOr("PAP_DB_PATH", "papsync.db")
	adminEmail := os.Getenv("PAP_ADMIN_EMAIL")
	adminPassword := os.Getenv("PAP_ADMIN_PASSWORD")
	wsName := envOr("PAP_WORKSPACE_NAME", "Team")
	jwtSecret := jwtSecretFromEnv()

	certPath := os.Getenv("PAP_TLS_CERT")
	keyPath := os.Getenv("PAP_TLS_KEY")
	behindProxy := os.Getenv("PAP_BEHIND_PROXY") == "1"

	mode, err := decideServeMode(certPath, keyPath, behindProxy)
	if err != nil {
		log.Fatal(err)
	}

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

	allowedOrigins := config.AllowedOriginsFromEnv()
	hub := ws.NewHub(jwtSecret, allowedOrigins)

	trustedProxyCIDRs, err := config.TrustedProxyCIDRsFromEnv()
	if err != nil {
		log.Fatal(err)
	}

	authHandler := &auth.Handler{
		DB:        db,
		JwtSecret: jwtSecret,
		TokenTTL:  24 * time.Hour,
	}

	syncH := &syncHandler.Handler{
		DB:  db,
		Hub: hub,
	}

	r := server.NewRouter(authHandler, syncH, hub, jwtSecret, server.Config{
		AllowedOrigins:       allowedOrigins,
		TLSAttivo:            mode == serveModeTLS,
		LoginRateLimit:       server.DefaultLoginRateLimit,
		LoginRateLimitWindow: server.DefaultLoginRateLimitWindow,
		Version:              "0.1.0",
		BehindProxy:          behindProxy,
		TrustedProxyCIDRs:    trustedProxyCIDRs,
	})

	log.Printf("PaP Sync Server avviato su :%s (modalità: %s)", port, mode)

	switch mode {
	case serveModeTLS:
		err = http.ListenAndServeTLS(":"+port, certPath, keyPath, r)
	case serveModeProxyHTTP:
		log.Printf("ATTENZIONE: server avviato in HTTP semplice (PAP_BEHIND_PROXY=1): " +
			"assicurati che il reverse proxy davanti termini il TLS")
		err = http.ListenAndServe(":"+port, r)
	}
	if err != nil {
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
	if secret == "" {
		b := make([]byte, minJwtSecretBytes)
		if _, err := rand.Read(b); err != nil {
			// Senza entropia valida non c'è un fallback sicuro: un
			// segreto JWT prevedibile permetterebbe di falsificare token
			// (CWE-330, Use of Insufficiently Random Values).
			log.Fatalf("Errore generazione PAP_JWT_SECRET casuale: %v", err)
		}
		generated := hex.EncodeToString(b)
		log.Printf("ATTENZIONE: PAP_JWT_SECRET non impostato, generato casualmente (non persistente tra riavvii)")
		return []byte(generated)
	}

	if err := validateJwtSecret(secret); err != nil {
		log.Fatal(err)
	}
	return []byte(secret)
}

func validateJwtSecret(secret string) error {
	if len(secret) < minJwtSecretBytes {
		return fmt.Errorf(
			"PAP_JWT_SECRET troppo corto (%d byte, minimo %d): un segreto debole rende i JWT falsificabili via brute-force",
			len(secret), minJwtSecretBytes)
	}
	return nil
}

// serveMode descrive come il server accetta connessioni in ingresso.
type serveMode string

const (
	// serveModeTLS: il server termina TLS direttamente con PAP_TLS_CERT /
	// PAP_TLS_KEY.
	serveModeTLS serveMode = "TLS diretto"
	// serveModeProxyHTTP: il server gira in HTTP semplice perché un
	// reverse proxy davanti (nginx/traefik/...) termina il TLS.
	serveModeProxyHTTP serveMode = "HTTP dietro reverse proxy"
)

// decideServeMode determina la modalità di ascolto del server, rifiutando
// di default l'avvio in HTTP in chiaro esposto direttamente (CWE-319,
// Cleartext Transmission of Sensitive Information: qui transitano
// credenziali e JWT). Il server accetta HTTP semplice solo se l'operatore
// dichiara esplicitamente che un reverse proxy davanti termina il TLS
// (PAP_BEHIND_PROXY=1).
func decideServeMode(certPath, keyPath string, behindProxy bool) (serveMode, error) {
	hasCert := certPath != ""
	hasKey := keyPath != ""

	if hasCert != hasKey {
		return "", fmt.Errorf("configurazione TLS incompleta: PAP_TLS_CERT e PAP_TLS_KEY vanno impostate entrambe o nessuna delle due")
	}
	if hasCert && hasKey {
		return serveModeTLS, nil
	}
	if behindProxy {
		return serveModeProxyHTTP, nil
	}
	return "", fmt.Errorf(
		"nessun certificato TLS configurato (PAP_TLS_CERT/PAP_TLS_KEY) e PAP_BEHIND_PROXY non impostato a 1: " +
			"il server rifiuta di avviarsi in HTTP in chiaro esposto direttamente. " +
			"Configura TLS oppure imposta PAP_BEHIND_PROXY=1 se un reverse proxy termina TLS davanti al server")
}
