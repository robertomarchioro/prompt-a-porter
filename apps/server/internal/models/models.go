package models

import "time"

type Workspace struct {
	Id          string  `json:"id"`
	Name        string  `json:"name"`
	Type        string  `json:"type"`
	ServerUrl   *string `json:"serverUrl,omitempty"`
	AccentColor *string `json:"accentColor,omitempty"`
	CreatedAt   string  `json:"createdAt"`
	UpdatedAt   string  `json:"updatedAt"`
	DeletedAt   *string `json:"deletedAt,omitempty"`
}

type User struct {
	Id           string  `json:"id"`
	WorkspaceId  string  `json:"workspaceId"`
	Email        string  `json:"email"`
	DisplayName  string  `json:"displayName"`
	Role         string  `json:"role"`
	PasswordHash string  `json:"-"`
	CreatedAt    string  `json:"createdAt"`
	UpdatedAt    string  `json:"updatedAt"`
	DeletedAt    *string `json:"deletedAt,omitempty"`
}

type Prompt struct {
	Id              string  `json:"id"`
	WorkspaceId     string  `json:"workspaceId"`
	AuthorUserId    string  `json:"authorUserId"`
	Title           string  `json:"title"`
	Description     *string `json:"description,omitempty"`
	Body            string  `json:"body"`
	Visibility      string  `json:"visibility"`
	TargetModel     *string `json:"targetModel,omitempty"`
	IsFavorite      int     `json:"isFavorite"`
	UseCount        int     `json:"useCount"`
	LastUsedAt      *string `json:"lastUsedAt,omitempty"`
	Version         int     `json:"version"`
	CreatedAt       string  `json:"createdAt"`
	UpdatedAt       string  `json:"updatedAt"`
	UpdatedByUserId *string `json:"updatedByUserId,omitempty"`
	DeletedAt       *string `json:"deletedAt,omitempty"`
}

type Tag struct {
	Id          string  `json:"id"`
	WorkspaceId string  `json:"workspaceId"`
	Name        string  `json:"name"`
	Color       *string `json:"color,omitempty"`
	CreatedAt   string  `json:"createdAt"`
	UpdatedAt   string  `json:"updatedAt"`
	DeletedAt   *string `json:"deletedAt,omitempty"`
}

type PromptTag struct {
	PromptId string `json:"promptId"`
	TagId    string `json:"tagId"`
}

type AuditEntry struct {
	Id          string  `json:"id"`
	WorkspaceId string  `json:"workspaceId"`
	UserId      *string `json:"userId,omitempty"`
	Action      string  `json:"action"`
	EntityType  string  `json:"entityType"`
	EntityId    *string `json:"entityId,omitempty"`
	Metadata    *string `json:"metadata,omitempty"`
	OccurredAt  string  `json:"occurredAt"`
}

type LoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

type LoginResponse struct {
	Token     string `json:"token"`
	ExpiresAt int64  `json:"expiresAt"`
	User      User   `json:"user"`
}

type SyncPullRequest struct {
	Since string `json:"since"`
}

type SyncDelta struct {
	Prompts    []Prompt    `json:"prompts"`
	Tags       []Tag       `json:"tags"`
	PromptTags []PromptTag `json:"promptTags"`
	Timestamp  string      `json:"timestamp"`
}

type SyncPushRequest struct {
	Prompts    []Prompt    `json:"prompts"`
	Tags       []Tag       `json:"tags"`
	PromptTags []PromptTag `json:"promptTags"`
}

type SyncPushResponse struct {
	Accepted  int    `json:"accepted"`
	Conflicts int    `json:"conflicts"`
	Timestamp string `json:"timestamp"`
}

type WsMessage struct {
	Type        string `json:"type"`
	WorkspaceId string `json:"workspaceId"`
	EntityType  string `json:"entityType,omitempty"`
	EntityId    string `json:"entityId,omitempty"`
	Timestamp   string `json:"timestamp"`
}

func NowUTC() string {
	return time.Now().UTC().Format("2006-01-02 15:04:05")
}
