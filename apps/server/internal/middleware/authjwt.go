package middleware

import (
	"net/http"
	"strings"

	"github.com/golang-jwt/jwt/v5"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/auth"
)

func JwtAuth(secret []byte) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			header := r.Header.Get("Authorization")
			if header == "" {
				http.Error(w, `{"error":"token mancante"}`, http.StatusUnauthorized)
				return
			}

			tokenStr := strings.TrimPrefix(header, "Bearer ")
			if tokenStr == header {
				http.Error(w, `{"error":"formato Authorization non valido"}`, http.StatusUnauthorized)
				return
			}

			claims := &auth.Claims{}
			// jwt.WithValidMethods forza l'algoritmo HS256: senza questo
			// vincolo un token con alg="none" o firmato con un algoritmo
			// diverso da quello atteso potrebbe bypassare la verifica
			// (CWE-347, algorithm confusion / JWT alg=none attack).
			token, err := jwt.ParseWithClaims(tokenStr, claims, func(t *jwt.Token) (any, error) {
				return secret, nil
			}, jwt.WithValidMethods([]string{"HS256"}))
			if err != nil || !token.Valid {
				http.Error(w, `{"error":"token non valido o scaduto"}`, http.StatusUnauthorized)
				return
			}

			ctx := auth.ContextWithClaims(r.Context(), claims)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}
