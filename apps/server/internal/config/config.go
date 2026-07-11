// Package config raccoglie helper di configurazione condivisi da più
// componenti del server (router HTTP, hub WebSocket) letti dalle variabili
// d'ambiente.
package config

import (
	"fmt"
	"log"
	"net/netip"
	"net/url"
	"os"
	"strings"
)

// AllowedOriginsFromEnv legge PAP_ALLOWED_ORIGINS (lista CSV di origin, es.
// "https://app.example.com,https://altro.example.com") e restituisce la
// allow-list da usare per CORS e per il controllo Origin del WebSocket.
//
// Default: nessuna origine browser consentita. L'assunzione è che il
// client desktop (Tauri) non sia soggetto a CORS/Origin per le chiamate
// HTTP/WS verso il server sync, ma dipende dal webview di sistema
// (WebView2/WebKitGTK/WKWebView) e NON è verificata in questo codice: prima
// di abilitare questa allow-list in produzione, controllare l'header
// Origin effettivo inviato dal client (vedi docs/operativo/
// deploy-produzione.md, sezione Sicurezza) e includerlo esplicitamente se
// presente, altrimenti si rischia di rifiutare il client desktop stesso.
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
			origins = append(origins, normalizeOrigin(o))
		}
	}
	return origins
}

// normalizeOrigin ripulisce una entry di PAP_ALLOWED_ORIGINS: toglie uno
// slash finale (un'Origin non ha mai un path, ma è un errore di
// configurazione facile da fare copiando un URL) e riporta schema e host in
// minuscolo (case-insensitive, coerente con come i browser serializzano
// sempre l'header Origin — RFC 6454 §6.1/§6.2). Logga un warning se
// l'origine non ha uno schema/host validi o se contiene un path, così un
// errore di configurazione che farebbe fallire silenziosamente ogni
// confronto con l'header Origin reale si vede nei log all'avvio.
func normalizeOrigin(raw string) string {
	trimmed := strings.TrimRight(raw, "/")

	u, err := url.Parse(trimmed)
	if err != nil || u.Scheme == "" || u.Host == "" {
		log.Printf("ATTENZIONE: PAP_ALLOWED_ORIGINS contiene %q che non sembra una origin valida "+
			"(manca lo schema o l'host, es. \"https://app.example.com\"): verrà usata così com'è, "+
			"ma probabilmente non corrisponderà mai all'header Origin inviato dal browser", raw)
		return trimmed
	}
	if u.Path != "" && u.Path != "/" {
		log.Printf("ATTENZIONE: PAP_ALLOWED_ORIGINS contiene %q con un path dopo l'host (%q): "+
			"una Origin valida è solo schema://host[:porta], il path non verrà mai confrontato "+
			"dal browser e viene ignorato qui", raw, u.Path)
	}

	return strings.ToLower(u.Scheme) + "://" + strings.ToLower(u.Host)
}

// OriginAllowed verifica se origin è presente nella allow-list. Il
// confronto è una uguaglianza esatta di stringa: è comunque
// effettivamente case-insensitive su schema/host perché sia
// AllowedOriginsFromEnv (via normalizeOrigin) sia i browser (RFC 6454)
// serializzano schema e host dell'Origin sempre in minuscolo.
func OriginAllowed(origin string, allowed []string) bool {
	for _, a := range allowed {
		if a == origin {
			return true
		}
	}
	return false
}

// TrustedProxyCIDRsFromEnv legge PAP_TRUSTED_PROXY_CIDR (lista CSV di
// prefissi CIDR, es. "10.0.0.0/8,172.16.0.0/12") usata per risolvere l'IP
// reale del client dall'header X-Forwarded-For quando il server gira dietro
// un reverse proxy (PAP_BEHIND_PROXY=1). Senza una lista esplicita di hop
// fidati, fidarsi ciecamente di X-Forwarded-For permetterebbe a un client di
// falsificare il proprio IP (CWE-290) e quindi di bypassare il rate-limit
// per-IP su /auth/login.
//
// Ritorna errore se un prefisso non è un CIDR valido, per far fallire
// l'avvio in modo esplicito (log.Fatal nel chiamante) invece di lasciare
// che chi/middleware.ClientIPFromXFF vada in panic più a valle.
func TrustedProxyCIDRsFromEnv() ([]string, error) {
	raw := os.Getenv("PAP_TRUSTED_PROXY_CIDR")
	if raw == "" {
		return nil, nil
	}

	parts := strings.Split(raw, ",")
	cidrs := make([]string, 0, len(parts))
	for _, p := range parts {
		c := strings.TrimSpace(p)
		if c == "" {
			continue
		}
		if _, err := netip.ParsePrefix(c); err != nil {
			return nil, fmt.Errorf("PAP_TRUSTED_PROXY_CIDR contiene un prefisso CIDR non valido %q: %w", c, err)
		}
		cidrs = append(cidrs, c)
	}
	return cidrs, nil
}
