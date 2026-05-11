package ws

import (
	"encoding/json"
	"log"
	"net/http"
	"strings"
	"sync"

	"github.com/golang-jwt/jwt/v5"
	"github.com/gorilla/websocket"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/auth"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/models"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

type client struct {
	conn        *websocket.Conn
	workspaceId string
}

type Hub struct {
	mu        sync.RWMutex
	clients   map[*client]struct{}
	jwtSecret []byte
}

func NewHub(jwtSecret []byte) *Hub {
	return &Hub{
		clients:   make(map[*client]struct{}),
		jwtSecret: jwtSecret,
	}
}

func (h *Hub) HandleWs(w http.ResponseWriter, r *http.Request) {
	tokenStr := r.URL.Query().Get("token")
	if tokenStr == "" {
		header := r.Header.Get("Authorization")
		tokenStr = strings.TrimPrefix(header, "Bearer ")
		if tokenStr == header {
			http.Error(w, `{"error":"token mancante"}`, http.StatusUnauthorized)
			return
		}
	}

	claims := &auth.Claims{}
	token, err := jwt.ParseWithClaims(tokenStr, claims, func(t *jwt.Token) (any, error) {
		return h.jwtSecret, nil
	})
	if err != nil || !token.Valid {
		http.Error(w, `{"error":"token non valido"}`, http.StatusUnauthorized)
		return
	}

	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("Errore upgrade WebSocket: %v", err)
		return
	}

	c := &client{conn: conn, workspaceId: claims.WorkspaceId}
	h.mu.Lock()
	h.clients[c] = struct{}{}
	h.mu.Unlock()

	log.Printf("WS connesso: user=%s workspace=%s", claims.UserId, claims.WorkspaceId)

	defer func() {
		h.mu.Lock()
		delete(h.clients, c)
		h.mu.Unlock()
		conn.Close()
		log.Printf("WS disconnesso: user=%s", claims.UserId)
	}()

	for {
		_, _, err := conn.ReadMessage()
		if err != nil {
			break
		}
	}
}

func (h *Hub) Broadcast(workspaceId string, msg models.WsMessage) {
	data, err := json.Marshal(msg)
	if err != nil {
		return
	}

	h.mu.RLock()
	defer h.mu.RUnlock()

	for c := range h.clients {
		if c.workspaceId == workspaceId {
			if err := c.conn.WriteMessage(websocket.TextMessage, data); err != nil {
				log.Printf("Errore invio WS: %v", err)
			}
		}
	}
}

func (h *Hub) ClientCount() int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	return len(h.clients)
}
