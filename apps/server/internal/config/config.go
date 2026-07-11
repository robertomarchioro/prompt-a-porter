// Package config raccoglie helper di configurazione condivisi da più
// componenti del server (router HTTP, hub WebSocket) letti dalle variabili
// d'ambiente.
package config

import (
	"os"
	"strings"
)

// AllowedOriginsFromEnv legge PAP_ALLOWED_ORIGINS (lista CSV di origin, es.
// "https://app.example.com,https://altro.example.com") e restituisce la
// allow-list da usare per CORS e per il controllo Origin del WebSocket.
//
// Default: nessuna origine browser consentita. Il client desktop (Tauri) non
// passa da CORS/Origin per le chiamate HTTP/WS verso il server sync, quindi
// questa allow-list serve solo per abilitare esplicitamente eventuali client
// browser (dashboard, integrazioni) che non esistono ancora.
func AllowedOriginsFromEnv() []string {
	raw := os.Getenv("PAP_ALLOWED_ORIGINS")
	if raw == "" {
		return []string{}
	}

	parts := strings.Split(raw, ",")
	origins := make([]string, 0, len(parts))
	for _, p := range parts {
		o := strings.TrimSpace(p)
		if o != "" {
			origins = append(origins, o)
		}
	}
	return origins
}

// OriginAllowed verifica se origin è presente nella allow-list (match
// esatto, case-sensitive come da RFC 6454).
func OriginAllowed(origin string, allowed []string) bool {
	for _, a := range allowed {
		if a == origin {
			return true
		}
	}
	return false
}
