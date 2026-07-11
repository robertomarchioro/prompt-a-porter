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
