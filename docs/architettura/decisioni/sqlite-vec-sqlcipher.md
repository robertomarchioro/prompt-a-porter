# Spike 1 — sqlite-vec ⊕ SQLCipher

> **Stato**: ✅ **PASSED** (eseguito 2026-05-04 su Ubuntu Linux x86_64, Rust stable, SQLCipher 4.5.7 community, sqlite-vec v0.1.9).
>
> **Verdict**: sqlite-vec è utilizzabile direttamente dal client desktop con la configurazione di produzione (`rusqlite 0.32 + bundled-sqlcipher-vendored-openssl`) tramite registrazione come auto-extension statico. **Step 2 di Fase 3 può procedere senza fallback architetturali.**

## Contesto

Fase 3 Step 2 prevede `CREATE VIRTUAL TABLE PromptsEmbeddings USING vec0(...)` per memorizzare embeddings da 384 dim. La tabella deve coesistere con il resto dello schema vault, che è cifrato SQLCipher (Argon2id → AES-256). Due rischi tecnici:

1. **Loadable extensions disabilitate**: SQLCipher tipicamente compila SQLite con `SQLITE_OMIT_LOAD_EXTENSION` o richiede esplicitamente `enable_load_extension(true)`, per ragioni di hardening.
2. **Symbol conflict**: SQLCipher fork rinomina alcune funzioni di SQLite. sqlite-vec potrebbe non trovare i simboli giusti.

## Strategia testata

**Auto-extension statico** (no loadable extension binari):

```rust
unsafe {
    let init: unsafe extern "C" fn() = std::mem::transmute(
        sqlite_vec::sqlite3_vec_init as *const ()
    );
    rusqlite::ffi::sqlite3_auto_extension(Some(init));
}
```

Il crate `sqlite-vec` (crates.io 0.1.x) vendor-izza il sorgente C, quindi il simbolo `sqlite3_vec_init` viene linkato direttamente nel binario. Registrandolo come *auto-extension* prima di aprire connessioni, si evita il path `db.load_extension()` (quello che SQLCipher potrebbe bloccare).

## Stage testati

Lo spike è suddiviso in 6 stage indipendenti per isolare il primo punto di rottura:

| Stage | Cosa verifica | Cosa significa il fallimento |
|---|---|---|
| 1 | `sqlite3_auto_extension` ritorna `SQLITE_OK` | API non disponibile in SQLCipher build |
| 2 | Apertura DB cifrato + `PRAGMA cipher_version` | SQLCipher non attivo |
| 3 | `SELECT vec_version()` | sqlite-vec init non eseguita su nuova connessione |
| 4 | `CREATE VIRTUAL TABLE ... USING vec0(...)` | vec0 vtable rifiutata da DB cifrato |
| 5 | INSERT di embedding f32 little-endian | encoding o storage problem |
| 6 | `embedding MATCH ?` + ordinamento per `distance` | KNN engine non funzionante su pages cifrate |

Il dataset è minimo (3 embeddings di dim 4) — l'obiettivo è binario PASS/FAIL, non benchmark.

## Come eseguire

```bash
# 1. Installa rustup se assente (idempotente, no sudo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

# 2. Carica path
. "$HOME/.cargo/env"

# 3. Esegui spike
cd /home/roberto/prompt-a-porter/spikes/sqlite-vec
cargo run --release
```

Prima esecuzione: ~3-5 min (compila SQLCipher + OpenSSL vendor + sqlite-vec). Successive: <1 s.

## Risultati (esecuzione 2026-05-04)

```
=== Spike sqlite-vec ⊕ SQLCipher ===

[1/6] Registro sqlite_vec_init come auto-extension...
    ok
[2/6] Creo vault SQLCipher cifrato in /tmp/spike-sqlite-vec-vault.db...
    SQLCipher attivo, versione: 4.5.7 community
    sanity check tabella standard: ok
[3/6] Interrogo vec_version()...
    sqlite-vec versione: v0.1.9
[4/6] CREATE VIRTUAL TABLE prompts_emb USING vec0(...) su DB cifrato...
    ok
[5/6] INSERT di 3 embeddings di dimensione 4...
    inseriti 3 righe
[6/6] Query nearest-neighbor con embedding MATCH...
    risultati (3):
      prm-2-email-informale → distance 0.0200
      prm-1-email-formale → distance 0.0200
      prm-3-recipe-pasta → distance 0.9147
    ranking semantico coerente: top = prm-2-email-informale

✅ SPIKE PASSED
```

### Diagnosi

Tutti e 6 gli stage chiusi senza intoppi:
- L'auto-extension API è **disponibile** in SQLCipher 4.5.7 community (smentisce il timore che fosse compilata via `SQLITE_OMIT_LOAD_EXTENSION`).
- `sqlite-vec` v0.1.9 vendor-izzato come crate Rust si linka staticamente alla build SQLCipher senza conflitti di simbolo.
- `vec0` virtual table convive col layout cifrato senza richiedere flag aggiuntivi: la cifratura SQLCipher opera a livello di pagine, e le pagine della vtable vec0 sono cifrate come tutte le altre.
- INSERT/MATCH/distance funzionano end-to-end. Le distanze L2 ottenute sono semanticamente coerenti col dataset (prm-1 e prm-2, embedding vicini, hanno distance bassa identica; prm-3 lontano in spazio vettoriale ha distance alta).

Nessun fallback necessario.

## Decisione

✅ **Step 2 di Fase 3 procede col path originale**: `CREATE VIRTUAL TABLE PromptsEmbeddings USING vec0(PromptId TEXT PRIMARY KEY, Embedding FLOAT[384])` dentro il vault SQLCipher esistente.

Niente file SQLite separato, niente build custom di SQLCipher, niente switch di driver. Le pagine della vtable vengono cifrate insieme al resto del vault — privacy preservata.

### Implementazione di produzione (per Step 2)

Replicare il pattern dello spike nel client desktop:

1. In `apps/client/src-tauri/Cargo.toml`: aggiungere `sqlite-vec = "0.1"` come dependency.
2. In `apps/client/src-tauri/src/vault.rs` (o equivalente bootstrap): chiamare `sqlite3_auto_extension(sqlite3_vec_init)` **una sola volta**, prima della prima `Connection::open`. Idempotente — se già registrato, ritorna comunque `SQLITE_OK`.
3. Migration v3 con `CREATE VIRTUAL TABLE PromptsEmbeddings USING vec0(...)` e indici/hooks come da Step 2.

Vedi `spikes/sqlite-vec/src/main.rs` per il pattern esatto del transmute della funzione init (necessario perché il binding Rust di sqlite-vec espone una signature semplificata).

## Riferimenti

- sqlite-vec: <https://github.com/asg017/sqlite-vec>
- sqlite-vec Rust binding: <https://github.com/asg017/sqlite-vec/tree/main/bindings/rust>
- rusqlite SQLCipher: <https://docs.rs/rusqlite/latest/rusqlite/#features>
- SQLCipher security model: <https://www.zetetic.net/sqlcipher/design/>
