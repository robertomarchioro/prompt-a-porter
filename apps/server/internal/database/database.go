package database

import (
	"crypto/rand"
	"database/sql"
	_ "embed"
	"encoding/hex"
	"fmt"
	"log"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"golang.org/x/crypto/argon2"

	"github.com/anthropics/prompt-a-porter/apps/server/internal/models"
)

//go:embed schema.sql
var schemaSql string

type DB struct {
	*sql.DB
}

func Open(path string) (*DB, error) {
	db, err := sql.Open("sqlite3", path+"?_journal_mode=WAL&_foreign_keys=ON")
	if err != nil {
		return nil, fmt.Errorf("apertura db: %w", err)
	}
	db.SetMaxOpenConns(1)
	return &DB{db}, nil
}

func (db *DB) Migrate() error {
	if _, err := db.Exec(schemaSql); err != nil {
		return fmt.Errorf("migrazione: %w", err)
	}

	var count int
	err := db.QueryRow("SELECT COUNT(*) FROM _Migrazioni WHERE Versione = '001'").Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		_, err = db.Exec("INSERT INTO _Migrazioni (Versione, ApplicataIl) VALUES ('001', ?)",
			models.NowUTC())
		if err != nil {
			return err
		}
	}
	return nil
}

func (db *DB) SeedAdmin(email, password, workspaceName string) error {
	var count int
	if err := db.QueryRow("SELECT COUNT(*) FROM Users WHERE Email = ?", email).Scan(&count); err != nil {
		return err
	}
	if count > 0 {
		return nil
	}

	now := models.NowUTC()
	wsId := GeneraId("ws")
	userId := GeneraId("usr")

	hash, err := HashPassword(password)
	if err != nil {
		return fmt.Errorf("hash password admin: %w", err)
	}

	tx, err := db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	_, err = tx.Exec(`INSERT INTO Workspaces (Id, Name, Type, CreatedAt, UpdatedAt)
		VALUES (?, ?, 'team', ?, ?)`, wsId, workspaceName, now, now)
	if err != nil {
		return err
	}

	_, err = tx.Exec(`INSERT INTO Users (Id, WorkspaceId, Email, DisplayName, Role, PasswordHash, CreatedAt, UpdatedAt)
		VALUES (?, ?, ?, ?, 'Admin', ?, ?, ?)`, userId, wsId, email, "Admin", hash, now, now)
	if err != nil {
		return err
	}

	log.Printf("Admin creato: %s (workspace: %s)", email, workspaceName)
	return tx.Commit()
}

func GeneraId(prefix string) string {
	ts := time.Now().UnixMilli()
	rnd := make([]byte, 4)
	rand.Read(rnd)
	return fmt.Sprintf("%s-%012x%s", prefix, ts, hex.EncodeToString(rnd))
}

func HashPassword(password string) (string, error) {
	salt := make([]byte, 16)
	if _, err := rand.Read(salt); err != nil {
		return "", err
	}
	hash := argon2.IDKey([]byte(password), salt, 3, 64*1024, 4, 32)
	return fmt.Sprintf("$argon2id$v=19$m=65536,t=3,p=4$%s$%s",
		hex.EncodeToString(salt),
		hex.EncodeToString(hash)), nil
}

func VerifyPassword(password, encoded string) bool {
	var salt, hash []byte
	var m uint32
	var t uint32
	var p uint8

	_, err := fmt.Sscanf(encoded, "$argon2id$v=19$m=%d,t=%d,p=%d$", &m, &t, &p)
	if err != nil {
		return false
	}

	parts := splitArgon2(encoded)
	if len(parts) != 2 {
		return false
	}

	salt, err = hex.DecodeString(parts[0])
	if err != nil {
		return false
	}
	hash, err = hex.DecodeString(parts[1])
	if err != nil {
		return false
	}

	computed := argon2.IDKey([]byte(password), salt, t, m, p, uint32(len(hash)))
	return constantTimeEqual(computed, hash)
}

func splitArgon2(encoded string) []string {
	count := 0
	lastDollar := -1
	var parts []string
	for i, c := range encoded {
		if c == '$' {
			count++
			if count >= 4 {
				if lastDollar >= 0 && count > 4 {
					parts = append(parts, encoded[lastDollar+1:i])
				}
				lastDollar = i
			}
		}
	}
	if lastDollar >= 0 && lastDollar < len(encoded)-1 {
		parts = append(parts, encoded[lastDollar+1:])
	}
	return parts
}

func constantTimeEqual(a, b []byte) bool {
	if len(a) != len(b) {
		return false
	}
	var result byte
	for i := range a {
		result |= a[i] ^ b[i]
	}
	return result == 0
}
