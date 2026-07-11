package middleware

import "net/http"

// SecurityHeaders aggiunge header di difesa in profondità a tutte le
// risposte del server. HSTS viene impostato solo quando tlsAttivo è true
// (il server serve direttamente TLS): se invece il server gira in HTTP
// semplice dietro un reverse proxy (PAP_BEHIND_PROXY=1), è compito del
// proxy che termina il TLS impostare Strict-Transport-Security.
func SecurityHeaders(tlsAttivo bool) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("X-Content-Type-Options", "nosniff")
			w.Header().Set("X-Frame-Options", "DENY")
			if tlsAttivo {
				w.Header().Set("Strict-Transport-Security", "max-age=31536000; includeSubDomains")
			}
			next.ServeHTTP(w, r)
		})
	}
}
