package middleware

import (
	"bufio"
	"errors"
	"log"
	"net"
	"net/http"
	"time"
)

type wrappedWriter struct {
	http.ResponseWriter
	statusCode int
}

func (w *wrappedWriter) WriteHeader(code int) {
	w.statusCode = code
	w.ResponseWriter.WriteHeader(code)
}

// Hijack inoltra all'http.Hijacker sottostante. Senza questo metodo
// wrappedWriter non implementa http.Hijacker (l'interfaccia embeddata
// http.ResponseWriter non promuove Hijack), e qualunque handler che debba
// "prendere in mano" la connessione — come l'upgrade WebSocket di
// gorilla/websocket su /ws — fallirebbe con "response does not implement
// http.Hijacker" ogni volta che passa da questo middleware.
func (w *wrappedWriter) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	hijacker, ok := w.ResponseWriter.(http.Hijacker)
	if !ok {
		return nil, nil, errors.New("il ResponseWriter sottostante non supporta l'hijacking")
	}
	return hijacker.Hijack()
}

func Logger(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		wrapped := &wrappedWriter{ResponseWriter: w, statusCode: http.StatusOK}
		next.ServeHTTP(wrapped, r)
		log.Printf("%s %s %d %s", r.Method, r.URL.Path, wrapped.statusCode, time.Since(start).Round(time.Millisecond))
	})
}
