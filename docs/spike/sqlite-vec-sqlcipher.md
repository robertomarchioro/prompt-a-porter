# Spike 1 — sqlite-vec ⊕ SQLCipher

> **Stato**: in attesa di esecuzione (richiede toolchain Rust locale).
>
> **Domanda chiave**: sqlite-vec è utilizzabile dal client desktop, che apre il vault SQLite in modalità SQLCipher cifrata via `rusqlite 0.32 + bundled-sqlcipher-vendored-openssl`?
>
> **Output atteso**: PASS / FAIL con dettaglio del primo stage che fallisce, e — in caso di FAIL — quale fallback adottare.

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

## Risultati

> _Da compilare dopo l'esecuzione._

```
[output di cargo run qui]
```

### Diagnosi

> _Verdict + raccomandazione._

## Decisione

> _PASS_ → si procede con Step 2 originale (vec0 dentro vault SQLCipher).
>
> _FAIL stage 1-3_ → fallback A: vec0 in **file SQLite separato non cifrato** (path `~/.local/share/prompt-a-porter/embeddings.db`). Privacy implication: gli embeddings sono float "rumorosi" da cui non si recupera il prompt originale, ma il path → prompt id è leakabile. Da documentare in `docs/ricerca-semantica.md`.
>
> _FAIL stage 4-6_ → fallback B: build custom di rusqlite con SQLCipher patched, o switch driver. Costo alto, decidere col maintainer.

## Riferimenti

- sqlite-vec: <https://github.com/asg017/sqlite-vec>
- sqlite-vec Rust binding: <https://github.com/asg017/sqlite-vec/tree/main/bindings/rust>
- rusqlite SQLCipher: <https://docs.rs/rusqlite/latest/rusqlite/#features>
- SQLCipher security model: <https://www.zetetic.net/sqlcipher/design/>
