package ws

import (
	"encoding/json"
	"log"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/gorilla/websocket"

	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/auth"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/config"
	"github.com/robertomarchioro/prompt-a-porter/apps/server/internal/models"
)

const (
	// maxWsMessageBytes limita la dimensione dei messaggi in ingresso dal
	// client: il protocollo attuale non prevede payload dal client (solo
	// ping/pong e chiusura), quindi un limite basso è sufficiente a
	// prevenire DoS da messaggi enormi (CWE-400).
	maxWsMessageBytes = 4096

	// wsPongWait è il tempo massimo di attesa di un pong prima di
	// considerare la connessione morta.
	wsPongWait = 60 * time.Second

	// wsPingPeriod deve essere minore di wsPongWait per dare tempo al
	// client di rispondere prima dello scadere del deadline.
	wsPingPeriod = (wsPongWait * 9) / 10

	// wsWriteWait è il timeout per ogni singola scrittura (ping o
	// broadcast) sulla connessione.
	wsWriteWait = 10 * time.Second
)

type client struct {
	conn        *websocket.Conn
	workspaceId string

	// writeMu serializza le scritture sulla connessione: gorilla/websocket
	// non supporta scritture concorrenti dalla stessa connessione, e sia il
	// ping periodico sia Broadcast (chiamato da goroutine di richieste HTTP
	// diverse) scrivono sullo stesso client.
	writeMu sync.Mutex
}

func (c *client) writeMessage(messageType int, data []byte) error {
	c.writeMu.Lock()
	defer c.writeMu.Unlock()
	c.conn.SetWriteDeadline(time.Now().Add(wsWriteWait))
	return c.conn.WriteMessage(messageType, data)
}

type Hub struct {
	mu             sync.RWMutex
	clients        map[*client]struct{}
	jwtSecret      []byte
	allowedOrigins []string
	upgrader       websocket.Upgrader
}

// NewHub crea un Hub WebSocket. allowedOrigins è la allow-list (stessa di
// PAP_ALLOWED_ORIGINS usata per CORS, vedi internal/config) usata da
// CheckOrigin per rifiutare connessioni WS avviate da pagine browser non
// autorizzate (CWE-346, Origin Validation Error / cross-site WebSocket
// hijacking). Le richieste senza header Origin (client non-browser, es. il
// client desktop Tauri o strumenti da riga di comando) sono sempre
// consentite: solo i browser inviano Origin in modo affidabile.
func NewHub(jwtSecret []byte, allowedOrigins []string) *Hub {
	h := &Hub{
		clients:        make(map[*client]struct{}),
		jwtSecret:      jwtSecret,
		allowedOrigins: allowedOrigins,
	}
	h.upgrader = websocket.Upgrader{
		CheckOrigin: h.checkOrigin,
	}
	return h
}

func (h *Hub) checkOrigin(r *http.Request) bool {
	origin := r.Header.Get("Origin")
	if origin == "" {
		return true
	}
	return config.OriginAllowed(origin, h.allowedOrigins)
}

// extractToken cerca il token JWT nella richiesta di upgrade WebSocket. In
// ordine di preferenza:
//  1. Header Sec-WebSocket-Protocol (il client invia il token come valore
//     del subprotocol, es. new WebSocket(url, [token])): è il metodo
//     raccomandato perché non finisce nei log di accesso/URL come farebbe
//     un query param.
//  2. Header Authorization: Bearer <token>.
//  3. Query param ?token=... (DEPRECATO): mantenuto solo perché il client
//     desktop attuale (apps/client/src/lib/sync.ts, connettiWs) si connette
//     ancora così. Va rimosso quando il client verrà aggiornato per usare
//     Sec-WebSocket-Protocol.
func extractToken(r *http.Request) string {
	if proto := r.Header.Get("Sec-WebSocket-Protocol"); proto != "" {
		candidate := strings.TrimSpace(strings.Split(proto, ",")[0])
		if candidate != "" {
			return candidate
		}
	}

	if header := r.Header.Get("Authorization"); header != "" {
		t := strings.TrimPrefix(header, "Bearer ")
		if t != header {
			return t
		}
	}

	return r.URL.Query().Get("token")
}

func (h *Hub) HandleWs(w http.ResponseWriter, r *http.Request) {
	tokenStr := extractToken(r)
	if tokenStr == "" {
		http.Error(w, `{"error":"token mancante"}`, http.StatusUnauthorized)
		return
	}

	claims := &auth.Claims{}
	// jwt.WithValidMethods forza HS256, vedi lo stesso vincolo in
	// internal/middleware/authjwt.go (CWE-347, algorithm confusion).
	token, err := jwt.ParseWithClaims(tokenStr, claims, func(t *jwt.Token) (any, error) {
		return h.jwtSecret, nil
	}, jwt.WithValidMethods([]string{"HS256"}))
	if err != nil || !token.Valid {
		http.Error(w, `{"error":"token non valido"}`, http.StatusUnauthorized)
		return
	}

	conn, err := h.upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("Errore upgrade WebSocket: %v", err)
		return
	}

	c := &client{conn: conn, workspaceId: claims.WorkspaceId}
	h.mu.Lock()
	h.clients[c] = struct{}{}
	h.mu.Unlock()

	log.Printf("WS connesso: user=%s workspace=%s", claims.UserId, claims.WorkspaceId)

	done := make(chan struct{})
	go h.pingLoop(c, done)

	defer func() {
		close(done)
		h.mu.Lock()
		delete(h.clients, c)
		h.mu.Unlock()
		conn.Close()
		log.Printf("WS disconnesso: user=%s", claims.UserId)
	}()

	conn.SetReadLimit(maxWsMessageBytes)
	conn.SetReadDeadline(time.Now().Add(wsPongWait))
	conn.SetPongHandler(func(string) error {
		conn.SetReadDeadline(time.Now().Add(wsPongWait))
		return nil
	})

	for {
		_, _, err := conn.ReadMessage()
		if err != nil {
			break
		}
	}
}

// pingLoop invia un ping periodico per tenere viva la connessione e
// rilevare client morti/disconnessi in modo pulito, così i client zombie
// non restano indefinitamente nella mappa clients.
func (h *Hub) pingLoop(c *client, done <-chan struct{}) {
	ticker := time.NewTicker(wsPingPeriod)
	defer ticker.Stop()

	for {
		select {
		case <-done:
			return
		case <-ticker.C:
			if err := c.writeMessage(websocket.PingMessage, nil); err != nil {
				return
			}
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
			if err := c.writeMessage(websocket.TextMessage, data); err != nil {
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
