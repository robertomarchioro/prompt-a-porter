package main

import (
	"net/http"
	"testing"
)

// G112/Slowloris: l'http.Server deve avere timeout espliciti, altrimenti un
// client può tenere aperte connessioni a oltranza (DoS pre-auth).
func TestNewHTTPServer_TimeoutImpostati(t *testing.T) {
	srv := newHTTPServer(":8443", http.NewServeMux())

	if srv.ReadHeaderTimeout == 0 {
		t.Fatal("ReadHeaderTimeout non impostato: espone a Slowloris (slow-header)")
	}
	if srv.ReadTimeout == 0 {
		t.Fatal("ReadTimeout non impostato: lettura richiesta illimitata")
	}
	if srv.IdleTimeout == 0 {
		t.Fatal("IdleTimeout non impostato: keep-alive idle illimitati")
	}
	if srv.Addr != ":8443" {
		t.Fatalf("Addr atteso :8443, ottenuto %q", srv.Addr)
	}
	if srv.Handler == nil {
		t.Fatal("Handler non impostato")
	}
}

func TestValidateJwtSecret_CortoRifiutato(t *testing.T) {
	if err := validateJwtSecret("troppo-corto"); err == nil {
		t.Fatal("un segreto di meno di 32 byte deve essere rifiutato")
	}
}

func TestValidateJwtSecret_LungoAccettato(t *testing.T) {
	secret := "questo-segreto-ha-esattamente-32-byte-e-anche-piu"
	if err := validateJwtSecret(secret); err != nil {
		t.Fatalf("un segreto di %d byte deve essere accettato, errore: %v", len(secret), err)
	}
}

func TestValidateJwtSecret_LimiteEsatto(t *testing.T) {
	secret32 := "12345678901234567890123456789012" // 33 byte, sopra il limite
	if err := validateJwtSecret(secret32); err != nil {
		t.Fatalf("segreto di %d byte deve essere accettato: %v", len(secret32), err)
	}

	secret31 := secret32[:31]
	if err := validateJwtSecret(secret31); err == nil {
		t.Fatalf("segreto di %d byte deve essere rifiutato", len(secret31))
	}
}

func TestDecideServeMode_ConCertificato(t *testing.T) {
	mode, err := decideServeMode("/etc/tls/cert.pem", "/etc/tls/key.pem", false)
	if err != nil {
		t.Fatalf("errore inatteso: %v", err)
	}
	if mode != serveModeTLS {
		t.Fatalf("atteso serveModeTLS, ottenuto %q", mode)
	}
}

func TestDecideServeMode_DietroProxySenzaCert(t *testing.T) {
	mode, err := decideServeMode("", "", true)
	if err != nil {
		t.Fatalf("errore inatteso: %v", err)
	}
	if mode != serveModeProxyHTTP {
		t.Fatalf("atteso serveModeProxyHTTP, ottenuto %q", mode)
	}
}

func TestDecideServeMode_SenzaCertNeProxy_Rifiutato(t *testing.T) {
	if _, err := decideServeMode("", "", false); err == nil {
		t.Fatal("senza TLS né PAP_BEHIND_PROXY il server deve rifiutarsi di avviarsi")
	}
}

func TestDecideServeMode_ConfigurazioneParzialeRifiutata(t *testing.T) {
	if _, err := decideServeMode("/etc/tls/cert.pem", "", true); err == nil {
		t.Fatal("solo PAP_TLS_CERT senza PAP_TLS_KEY deve essere rifiutato anche dietro proxy")
	}
	if _, err := decideServeMode("", "/etc/tls/key.pem", false); err == nil {
		t.Fatal("solo PAP_TLS_KEY senza PAP_TLS_CERT deve essere rifiutato")
	}
}
