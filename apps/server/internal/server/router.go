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

	// BehindProxy e TrustedProxyCIDRs specchiano PAP_BEHIND_PROXY e
	// PAP_TRUSTED_PROXY_CIDR: quando il server è dietro un reverse proxy,
	// il rate-limit su /auth/login deve leggere l'IP reale del client da
	// X-Forwarded-For (fidandosi solo dei prefissi indicati), altrimenti
	// r.RemoteAddr è l'IP del proxy e tutti gli utenti dietro quel proxy
	// condividerebbero lo stesso bucket (un utente che sbaglia password
	// blocca tutti gli altri).
	BehindProxy       bool
	TrustedProxyCIDRs []string
}

// NewRouter costruisce il router chi completo con tutte le rotte e i
// middleware di sicurezza applicati.
func NewRouter(authHandler *auth.Handler, syncH *syncHandler.Handler, hub *ws.Hub, jwtSecret []byte, cfg Config) *chi.Mux {
	r := chi.NewRouter()

	r.Use(chiMiddleware.Recoverer)
	r.Use(middleware.Logger)
	r.Use(middleware.SecurityHeaders(cfg.TLSAttivo))
	r.Use(cors.Handler(corsOptions(cfg.AllowedOrigins)))

	if cfg.BehindProxy {
		r.Use(chiMiddleware.ClientIPFromXFF(cfg.TrustedProxyCIDRs...))
	}

	r.Get("/health", healthHandler(cfg.Version))

	r.With(httprate.LimitBy(cfg.LoginRateLimit, cfg.LoginRateLimitWindow, loginRateLimitKeyFunc(cfg.BehindProxy))).
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

// loginRateLimitKeyFunc sceglie come derivare la chiave del rate-limiter
// per-IP su /auth/login. Non dietro proxy, r.RemoteAddr è l'IP reale del
// client: httprate.KeyByIP va bene. Dietro un reverse proxy, r.RemoteAddr è
// l'IP del proxy per ogni richiesta: usare comunque KeyByIP metterebbe
// tutti gli utenti nello stesso bucket (self-inflicted DoS, vedi il
// commento su Config.BehindProxy). In quel caso la chiave viene letta
// dall'IP risolto da chiMiddleware.ClientIPFromXFF (installato come
// middleware globale sopra quando BehindProxy è true).
func loginRateLimitKeyFunc(behindProxy bool) httprate.KeyFunc {
	if !behindProxy {
		return httprate.KeyByIP
	}
	return func(r *http.Request) (string, error) {
		return httprate.CanonicalizeIP(chiMiddleware.GetClientIP(r.Context())), nil
	}
}

// corsOptions costruisce le opzioni CORS a partire dalla allow-list. NON si
// può semplicemente passare allowedOrigins a cors.Options{AllowedOrigins:
// ...}: go-chi/cors tratta una AllowedOrigins vuota (nil o len==0) come
// "consenti tutte le origin" quando AllowOriginFunc è nil (vedi
// go-chi/cors@v1.2.2 New(), c.allowedOriginsAll = true), il che avrebbe
// silenziosamente vanificato il default sicuro "nessuna origine consentita"
// documentato per PAP_ALLOWED_ORIGINS non impostata. Quando la allow-list è
// vuota impostiamo esplicitamente un AllowOriginFunc che rifiuta sempre,
// per rendere il "deny all" di default un comportamento reale e non
// implicito/fragile.
func corsOptions(allowedOrigins []string) cors.Options {
	opts := cors.Options{
		AllowedMethods: []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders: []string{"Authorization", "Content-Type"},
		ExposedHeaders: []string{"Content-Length"},
		// Il client desktop autentica con Bearer token nell'header
		// Authorization, non con cookie: non servono credenziali cross-site
		// (CWE-942, Overly Permissive CORS via wildcard + credentials).
		AllowCredentials: false,
		MaxAge:           300,
	}
	if len(allowedOrigins) == 0 {
		opts.AllowOriginFunc = func(r *http.Request, origin string) bool { return false }
	} else {
		opts.AllowedOrigins = allowedOrigins
	}
	return opts
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
