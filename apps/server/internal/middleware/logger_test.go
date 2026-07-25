package middleware

import (
	"bytes"
	"log"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestSanitizzaPerLog_NeutralizzaControlli(t *testing.T) {
	in := "/\nLogin riuscito: admin@promptaporter.it\rX\x7f\tY"
	got := sanitizzaPerLog(in)

	if strings.ContainsAny(got, "\n\r\t") || strings.ContainsRune(got, 0x7f) {
		t.Fatalf("i caratteri di controllo non sono stati neutralizzati: %q", got)
	}
	want := "/?Login riuscito: admin@promptaporter.it?X??Y"
	if got != want {
		t.Fatalf("atteso %q, ottenuto %q", want, got)
	}
}

func TestSanitizzaPerLog_PreservaPathLegittimo(t *testing.T) {
	for _, p := range []string{"/auth/login", "/sync/pull", "/ws"} {
		if got := sanitizzaPerLog(p); got != p {
			t.Fatalf("path legittimo alterato: atteso %q, ottenuto %q", p, got)
		}
	}
}

// TestLogger_NonForgiaSecondaRiga verifica che un path con newline decodificato
// non produca una seconda riga di log (CRLF log forging, CWE-117): l'intera
// richiesta deve restare su un solo record.
func TestLogger_NonForgiaSecondaRiga(t *testing.T) {
	var buf bytes.Buffer
	orig := log.Writer()
	flags := log.Flags()
	prefix := log.Prefix()
	log.SetOutput(&buf)
	log.SetFlags(0)
	log.SetPrefix("")
	t.Cleanup(func() {
		log.SetOutput(orig)
		log.SetFlags(flags)
		log.SetPrefix(prefix)
	})

	handler := Logger(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/", nil)
	// Simula il path già percent-DECODED da net/http: contiene un vero newline.
	req.URL.Path = "/\nLogin riuscito: admin@promptaporter.it"
	handler.ServeHTTP(httptest.NewRecorder(), req)

	out := strings.TrimRight(buf.String(), "\n")
	if strings.Contains(out, "\n") {
		t.Fatalf("il middleware ha emesso più di una riga di log: %q", buf.String())
	}
	if strings.Contains(out, "Login riuscito") == false {
		t.Fatalf("il payload sanitizzato dovrebbe restare sulla stessa riga: %q", out)
	}
}
