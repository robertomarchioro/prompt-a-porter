// Package server assembla il router HTTP del sync server: è l'unico punto
// in cui vengono cablate insieme le rotte e le catene di middleware
// (sicurezza, CORS, rate-limit, autenticazione), così cmd/papsync/main.go e
// i test di integrazione condividono esattamente la stessa configurazione
// (nessuna divergenza tra "quello che gira in produzione" e "quello che
// viene testato").
package server

import (
	"fmt"
	"net/http"
	"time"

	"github.com/go-chi/chi/v5"
	chiMiddleware "github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"
	"github.com/go-chi/httprate"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/auth"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/middleware"
	syncHandler "github.com/robertomarchioro/prompt-a-porter/apps/server/internal/sync"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/ws"
)

// DefaultLoginRateLimit e DefaultLoginRateLimitWindow sono i parametri di
// produzione del rate-limiter su /auth/login: 5 tentativi al minuto per IP
// sono sufficienti per un utente reale (anche con qualche errore di
// battitura) ma bloccano un attacco a forza bruta.
const (
	DefaultLoginRateLimit       = 5
	DefaultLoginRateLimitWindow = time.Minute
)

// Config raccoglie i parametri necessari a costruire il router. Zero-value
// non è utilizzabile: Version deve rimanere in sync con il version numbers
// pubblicati altrove.
type Config struct {
	AllowedOrigins       []string
	TLSAttivo            bool
	LoginRateLimit       int
	LoginRateLimitWindow time.Duration
	Version              string
}

// NewRouter costruisce il router chi completo con tutte le rotte e i
// middleware di sicurezza applicati.
func NewRouter(authHandler *auth.Handler, syncH *syncHandler.Handler, hub *ws.Hub, jwtSecret []byte, cfg Config) *chi.Mux {
	r := chi.NewRouter()

	r.Use(chiMiddleware.Recoverer)
	r.Use(middleware.Logger)
	r.Use(middleware.SecurityHeaders(cfg.TLSAttivo))
	r.Use(cors.Handler(cors.Options{
		AllowedOrigins: cfg.AllowedOrigins,
		AllowedMethods: []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders: []string{"Authorization", "Content-Type"},
		ExposedHeaders: []string{"Content-Length"},
		// Il client desktop autentica con Bearer token nell'header
		// Authorization, non con cookie: non servono credenziali cross-site
		// (CWE-942, Overly Permissive CORS via wildcard + credentials).
		AllowCredentials: false,
		MaxAge:           300,
	}))

	r.Get("/health", healthHandler(cfg.Version))

	r.With(httprate.LimitBy(cfg.LoginRateLimit, cfg.LoginRateLimitWindow, httprate.KeyByIP)).
		Post("/auth/login", authHandler.Login)

	r.Group(func(r chi.Router) {
		r.Use(middleware.JwtAuth(jwtSecret))

		r.Post("/auth/refresh", authHandler.Refresh)
		r.Get("/sync/pull", syncH.Pull)
		r.Post("/sync/push", syncH.Push)
	})

	r.Get("/ws", hub.HandleWs)

	return r
}

// healthHandler risponde con lo stato del server. Non espone metriche come
// il numero di client connessi: quell'informazione era leggibile senza
// autenticazione e poteva essere usata per ricognizione (CWE-200,
// Information Exposure) o per verificare da remoto quando un attacco WS
// (es. flood di connessioni) sta avendo effetto.
func healthHandler(version string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		fmt.Fprintf(w, `{"status":"ok","version":%q}`, version)
	}
}
