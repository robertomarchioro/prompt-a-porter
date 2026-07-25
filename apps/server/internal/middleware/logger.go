package middleware

import (
	"bufio"
	"errors"
	"log"
	"net"
	"net/http"
	"strings"
	"time"
)

// sanitizzaPerLog neutralizza i caratteri di controllo C0 (rune < 0x20, che
// includono CR e LF) e DEL (0x7f) sostituendoli con '?', per impedire la
// log-injection / CRLF log forging (CWE-117). Va applicata a ogni valore
// derivato dalla richiesta prima di scriverlo nel log: ad esempio r.URL.Path
// è il path già percent-DECODED, quindi un %0a/%0d nel request target diventa
// un vero newline capace di forgiare una seconda riga di log.
func sanitizzaPerLog(s string) string {
	return strings.Map(func(r rune) rune {
		if r < 0x20 || r == 0x7f {
			return '?'
		}
		return r
	}, s)
}

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
		log.Printf("%s %s %d %s", r.Method, sanitizzaPerLog(r.URL.Path), wrapped.statusCode, time.Since(start).Round(time.Millisecond))
	})
}
