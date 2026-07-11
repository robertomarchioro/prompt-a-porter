package sync

import (
	"database/sql"
	"encoding/json"
	"log"
	"net/http"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/auth"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/database"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/httpx"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/models"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/ws"
)

// maxPushBodyBytes limita il body di /sync/push: un delta di sync può
// contenere molti prompt/tag, ma 10 MiB coprono ampiamente l'uso reale e
// impediscono a un client di esaurire memoria del server (CWE-400).
const maxPushBodyBytes = 10 << 20 // 10 MiB

type Handler struct {
	DB  *database.DB
	Hub *ws.Hub
}

func (h *Handler) Pull(w http.ResponseWriter, r *http.Request) {
	claims, ok := auth.ClaimsFromContext(r.Context())
	if !ok {
		http.Error(w, `{"error":"non autenticato"}`, http.StatusUnauthorized)
		return
	}

	since := r.URL.Query().Get("since")
	if since == "" {
		since = "1970-01-01 00:00:00"
	}

	delta, err := h.pullDelta(claims.WorkspaceId, since)
	if err != nil {
		log.Printf("Errore pull: %v", err)
		http.Error(w, `{"error":"errore interno"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(delta)
}

func (h *Handler) pullDelta(workspaceId, since string) (*models.SyncDelta, error) {
	delta := &models.SyncDelta{
		Prompts:    []models.Prompt{},
		Tags:       []models.Tag{},
		PromptTags: []models.PromptTag{},
		Timestamp:  models.NowUTC(),
	}

	rows, err := h.DB.Query(`
		SELECT Id, WorkspaceId, AuthorUserId, Title, Description, Body, Visibility,
		       TargetModel, IsFavorite, UseCount, LastUsedAt, Version,
		       CreatedAt, UpdatedAt, UpdatedByUserId, DeletedAt
		FROM Prompts
		WHERE WorkspaceId = ? AND UpdatedAt > ? AND Visibility = 'workspace'
		ORDER BY UpdatedAt`,
		workspaceId, since)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	for rows.Next() {
		var p models.Prompt
		err := rows.Scan(&p.Id, &p.WorkspaceId, &p.AuthorUserId, &p.Title,
			&p.Description, &p.Body, &p.Visibility, &p.TargetModel,
			&p.IsFavorite, &p.UseCount, &p.LastUsedAt, &p.Version,
			&p.CreatedAt, &p.UpdatedAt, &p.UpdatedByUserId, &p.DeletedAt)
		if err != nil {
			return nil, err
		}
		delta.Prompts = append(delta.Prompts, p)
	}

	tagRows, err := h.DB.Query(`
		SELECT Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt, DeletedAt
		FROM Tags
		WHERE WorkspaceId = ? AND UpdatedAt > ?
		ORDER BY UpdatedAt`,
		workspaceId, since)
	if err != nil {
		return nil, err
	}
	defer tagRows.Close()

	for tagRows.Next() {
		var t models.Tag
		err := tagRows.Scan(&t.Id, &t.WorkspaceId, &t.Name, &t.Color,
			&t.CreatedAt, &t.UpdatedAt, &t.DeletedAt)
		if err != nil {
			return nil, err
		}
		delta.Tags = append(delta.Tags, t)
	}

	ptRows, err := h.DB.Query(`
		SELECT pt.PromptId, pt.TagId
		FROM PromptTags pt
		JOIN Prompts p ON p.Id = pt.PromptId
		WHERE p.WorkspaceId = ? AND p.UpdatedAt > ?`,
		workspaceId, since)
	if err != nil {
		return nil, err
	}
	defer ptRows.Close()

	for ptRows.Next() {
		var pt models.PromptTag
		if err := ptRows.Scan(&pt.PromptId, &pt.TagId); err != nil {
			return nil, err
		}
		delta.PromptTags = append(delta.PromptTags, pt)
	}

	return delta, nil
}

func (h *Handler) Push(w http.ResponseWriter, r *http.Request) {
	claims, ok := auth.ClaimsFromContext(r.Context())
	if !ok {
		http.Error(w, `{"error":"non autenticato"}`, http.StatusUnauthorized)
		return
	}

	var req models.SyncPushRequest
	if !httpx.DecodeJSONLimited(w, r, maxPushBodyBytes, &req, `{"error":"richiesta non valida"}`) {
		return
	}

	accepted, conflicts, err := h.pushDelta(claims.WorkspaceId, claims.UserId, &req)
	if err != nil {
		log.Printf("Errore push: %v", err)
		http.Error(w, `{"error":"errore interno"}`, http.StatusInternalServerError)
		return
	}

	now := models.NowUTC()

	if accepted > 0 && h.Hub != nil {
		h.Hub.Broadcast(claims.WorkspaceId, models.WsMessage{
			Type:        "sync_update",
			WorkspaceId: claims.WorkspaceId,
			Timestamp:   now,
		})
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(models.SyncPushResponse{
		Accepted:  accepted,
		Conflicts: conflicts,
		Timestamp: now,
	})
}

func (h *Handler) pushDelta(workspaceId, userId string, req *models.SyncPushRequest) (int, int, error) {
	tx, err := h.DB.Begin()
	if err != nil {
		return 0, 0, err
	}
	defer tx.Rollback()

	accepted := 0
	conflicts := 0
	now := models.NowUTC()

	for _, tag := range req.Tags {
		if tag.WorkspaceId != workspaceId {
			continue
		}
		var existing sql.NullString
		err := tx.QueryRow("SELECT UpdatedAt FROM Tags WHERE Id = ?", tag.Id).Scan(&existing)
		if err == sql.ErrNoRows {
			_, err = tx.Exec(`INSERT INTO Tags (Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt, DeletedAt)
				VALUES (?, ?, ?, ?, ?, ?, ?)`,
				tag.Id, workspaceId, tag.Name, tag.Color, tag.CreatedAt, now, tag.DeletedAt)
			if err != nil {
				return 0, 0, err
			}
			accepted++
		} else if err == nil {
			if existing.String >= tag.UpdatedAt {
				conflicts++
				continue
			}
			_, err = tx.Exec(`UPDATE Tags SET Name=?, Color=?, UpdatedAt=?, DeletedAt=? WHERE Id=?`,
				tag.Name, tag.Color, now, tag.DeletedAt, tag.Id)
			if err != nil {
				return 0, 0, err
			}
			accepted++
		} else {
			return 0, 0, err
		}

		h.logChange(tx, workspaceId, "tag", tag.Id, "upsert", tag, userId)
	}

	for _, p := range req.Prompts {
		if p.WorkspaceId != workspaceId || p.Visibility != "workspace" {
			continue
		}
		var existingUpdated sql.NullString
		var existingAuthor sql.NullString
		err := tx.QueryRow("SELECT UpdatedAt, AuthorUserId FROM Prompts WHERE Id = ?", p.Id).
			Scan(&existingUpdated, &existingAuthor)
		if err == sql.ErrNoRows {
			// L'autore di un prompt nuovo è sempre l'utente autenticato che
			// esegue il push, mai il valore fornito dal client: senza
			// questo un client malevolo potrebbe attribuire la paternità a
			// un altro utente (authorship spoofing, CWE-290 Authentication
			// Bypass by Assuming a Trusted Role). Anche il changelog deve
			// riportare l'autore corretto, non quello del client.
			inserted := p
			inserted.AuthorUserId = userId
			_, err = tx.Exec(`INSERT INTO Prompts
				(Id, WorkspaceId, AuthorUserId, Title, Description, Body, Visibility,
				 TargetModel, IsFavorite, UseCount, LastUsedAt, Version,
				 CreatedAt, UpdatedAt, UpdatedByUserId, DeletedAt)
				VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)`,
				inserted.Id, workspaceId, inserted.AuthorUserId, inserted.Title, inserted.Description, inserted.Body,
				inserted.Visibility, inserted.TargetModel, inserted.IsFavorite, inserted.UseCount, inserted.LastUsedAt,
				inserted.Version, inserted.CreatedAt, now, userId, inserted.DeletedAt)
			if err != nil {
				return 0, 0, err
			}
			accepted++
			h.logChange(tx, workspaceId, "prompt", inserted.Id, "upsert", inserted, userId)
		} else if err == nil {
			if existingUpdated.String >= p.UpdatedAt {
				conflicts++
				continue
			}
			_, err = tx.Exec(`UPDATE Prompts SET
				Title=?, Description=?, Body=?, Visibility=?, TargetModel=?,
				IsFavorite=?, UseCount=?, LastUsedAt=?, Version=?,
				UpdatedAt=?, UpdatedByUserId=?, DeletedAt=?
				WHERE Id=?`,
				p.Title, p.Description, p.Body, p.Visibility, p.TargetModel,
				p.IsFavorite, p.UseCount, p.LastUsedAt, p.Version,
				now, userId, p.DeletedAt, p.Id)
			if err != nil {
				return 0, 0, err
			}
			accepted++

			// L'update non modifica AuthorUserId in DB (non è nella SET
			// sopra): il changelog deve riflettere l'autore reale esistente,
			// non quello (eventualmente spoofato) inviato dal client.
			logged := p
			logged.AuthorUserId = existingAuthor.String
			h.logChange(tx, workspaceId, "prompt", logged.Id, "upsert", logged, userId)
		} else {
			return 0, 0, err
		}
	}

	for _, pt := range req.PromptTags {
		// Verifica che sia il prompt sia il tag appartengano al workspace
		// del chiamante prima di creare l'associazione. Senza questo check
		// un client autenticato in un workspace potrebbe collegare prompt e
		// tag di altri workspace (CWE-639, Authorization Bypass Through
		// User-Controlled Key). Coerente con i controlli dei loop Tags e
		// Prompts sopra. La SELECT gira dentro la stessa tx, quindi vede
		// anche prompt/tag inseriti poco prima in questo stesso push.
		var promptWs, tagWs string
		if err := tx.QueryRow("SELECT WorkspaceId FROM Prompts WHERE Id = ?", pt.PromptId).Scan(&promptWs); err != nil {
			if err == sql.ErrNoRows {
				continue
			}
			return 0, 0, err
		}
		if err := tx.QueryRow("SELECT WorkspaceId FROM Tags WHERE Id = ?", pt.TagId).Scan(&tagWs); err != nil {
			if err == sql.ErrNoRows {
				continue
			}
			return 0, 0, err
		}
		if promptWs != workspaceId || tagWs != workspaceId {
			continue
		}

		_, err := tx.Exec(`INSERT OR REPLACE INTO PromptTags (PromptId, TagId) VALUES (?, ?)`,
			pt.PromptId, pt.TagId)
		if err != nil {
			return 0, 0, err
		}
	}

	if err := tx.Commit(); err != nil {
		return 0, 0, err
	}

	return accepted, conflicts, nil
}

func (h *Handler) logChange(tx *sql.Tx, wsId, entityType, entityId, action string, payload any, userId string) {
	data, _ := json.Marshal(payload)
	tx.Exec(`INSERT INTO SyncChangelog (WorkspaceId, EntityType, EntityId, Action, Payload, ChangedAt, ChangedBy)
		VALUES (?, ?, ?, ?, ?, ?, ?)`,
		wsId, entityType, entityId, action, string(data), models.NowUTC(), userId)
}
