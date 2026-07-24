FROM golang:1.25-alpine AS builder

RUN apk add --no-cache gcc musl-dev

WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=1 go build -ldflags="-s -w" -o /papsync ./cmd/papsync

FROM alpine:3.20

RUN apk add --no-cache ca-certificates tzdata
RUN adduser -D -h /app papsync

WORKDIR /app
COPY --from=builder /papsync .

USER papsync

EXPOSE 8443

ENV PAP_PORT=8443 \
    PAP_DB_PATH=/app/data/papsync.db

VOLUME ["/app/data"]

ENTRYPOINT ["./papsync"]
