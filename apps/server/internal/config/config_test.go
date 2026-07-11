package config_test

import (
	"reflect"
	"testing"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/config"
)

func TestAllowedOriginsFromEnv_VuotoDefault(t *testing.T) {
	t.Setenv("PAP_ALLOWED_ORIGINS", "")

	got := config.AllowedOriginsFromEnv()

	if len(got) != 0 {
		t.Fatalf("attesa allow-list vuota di default, ottenuta %v", got)
	}
}

func TestAllowedOriginsFromEnv_ParsaCsvETrimma(t *testing.T) {
	t.Setenv("PAP_ALLOWED_ORIGINS", "https://a.example.com, https://b.example.com ,,https://c.example.com")

	got := config.AllowedOriginsFromEnv()
	want := []string{"https://a.example.com", "https://b.example.com", "https://c.example.com"}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("atteso %v, ottenuto %v", want, got)
	}
}

func TestOriginAllowed(t *testing.T) {
	allowed := []string{"https://a.example.com", "https://b.example.com"}

	if !config.OriginAllowed("https://a.example.com", allowed) {
		t.Fatal("origin presente in allow-list deve essere consentita")
	}
	if config.OriginAllowed("https://evil.example.com", allowed) {
		t.Fatal("origin non presente in allow-list non deve essere consentita")
	}
}

// TestAllowedOriginsFromEnv_NormalizzaSlashFinaleECase verifica che
// normalizeOrigin tolga uno slash finale e riporti schema/host in
// minuscolo, così un'origine configurata con maiuscole/slash residuo non
// fallisce silenziosamente il confronto con l'header Origin del browser
// (sempre minuscolo per RFC 6454).
func TestAllowedOriginsFromEnv_NormalizzaSlashFinaleECase(t *testing.T) {
	t.Setenv("PAP_ALLOWED_ORIGINS", "HTTPS://App.Example.COM/")

	got := config.AllowedOriginsFromEnv()
	want := []string{"https://app.example.com"}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("atteso %v, ottenuto %v", want, got)
	}
}

func TestTrustedProxyCIDRsFromEnv_VuotoDefault(t *testing.T) {
	t.Setenv("PAP_TRUSTED_PROXY_CIDR", "")

	got, err := config.TrustedProxyCIDRsFromEnv()
	if err != nil {
		t.Fatalf("errore inatteso: %v", err)
	}
	if len(got) != 0 {
		t.Fatalf("attesa lista vuota di default, ottenuta %v", got)
	}
}

func TestTrustedProxyCIDRsFromEnv_ParsaCsv(t *testing.T) {
	t.Setenv("PAP_TRUSTED_PROXY_CIDR", "10.0.0.0/8, 172.16.0.0/12")

	got, err := config.TrustedProxyCIDRsFromEnv()
	if err != nil {
		t.Fatalf("errore inatteso: %v", err)
	}
	want := []string{"10.0.0.0/8", "172.16.0.0/12"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("atteso %v, ottenuto %v", want, got)
	}
}

func TestTrustedProxyCIDRsFromEnv_CidrInvalidoRitornaErrore(t *testing.T) {
	t.Setenv("PAP_TRUSTED_PROXY_CIDR", "non-un-cidr")

	if _, err := config.TrustedProxyCIDRsFromEnv(); err == nil {
		t.Fatal("un CIDR non valido deve produrre un errore, non un avvio silenzioso")
	}
}
