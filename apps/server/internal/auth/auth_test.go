package auth

import (
	"strings"
	"testing"
)

func TestSanitizzaPerLog(t *testing.T) {
	casi := []struct {
		nome   string
		in     string
		atteso string
	}{
		{"email legittima invariata", "mario@rossi.it", "mario@rossi.it"},
		{"stringa vuota", "", ""},
		{"newline sostituito", "x\n2026-07-24 Login riuscito: admin@corp", "x?2026-07-24 Login riuscito: admin@corp"},
		{"CR e LF sostituiti", "a\r\nb", "a??b"},
		{"tab e altri controlli sostituiti", "a\tb", "a?b"},
		{"DEL sostituito", "a\x7fb", "a?b"},
	}

	for _, c := range casi {
		t.Run(c.nome, func(t *testing.T) {
			got := sanitizzaPerLog(c.in)
			if got != c.atteso {
				t.Fatalf("sanitizzaPerLog(%q) = %q, atteso %q", c.in, got, c.atteso)
			}
		})
	}
}

// TestSanitizzaPerLogNonProduceRigheMultiple garantisce che nessun input,
// per quanto malevolo, possa introdurre un a-capo che spezzi una riga di log.
func TestSanitizzaPerLogNonProduceRigheMultiple(t *testing.T) {
	malevolo := "vittima@corp\r\n2026-07-24 12:00:00 Login riuscito: admin@corp\radmin"
	got := sanitizzaPerLog(malevolo)
	if strings.ContainsAny(got, "\r\n") {
		t.Fatalf("output contiene ancora CR/LF: %q", got)
	}
}
