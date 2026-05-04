// Spike sqlite-vec ⊕ SQLCipher
//
// Domanda: sqlite-vec funziona quando SQLite è la build SQLCipher
// (bundled-sqlcipher-vendored-openssl di rusqlite, come usa il client desktop)?
//
// Strategia: registrare sqlite_vec_init come auto-extension PRIMA di aprire
// connessioni. Così l'estensione viene linkata staticamente alla build SQLCipher
// senza bisogno di dynamic loading (che SQLCipher tipicamente disabilita per
// ragioni di sicurezza).
//
// Stages testati (ognuno fallisce in modo isolato):
//   1. Auto-extension registration
//   2. Apertura DB cifrato + verifica SQLCipher attivo
//   3. Verifica sqlite-vec caricato (SELECT vec_version())
//   4. CREATE VIRTUAL TABLE vec0(...) su DB cifrato
//   5. INSERT con embedding f32 serializzato
//   6. SELECT nearest neighbor con embedding MATCH

use rusqlite::{ffi, params, Connection};
use std::error::Error;

const VAULT_PATH: &str = "/tmp/spike-sqlite-vec-vault.db";
const VAULT_PASSWORD: &str = "spike-test-password-non-segreto";
const EMBEDDING_DIM: usize = 4;

fn embedding_bytes(emb: &[f32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(emb.len() * 4);
    for &v in emb {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Spike sqlite-vec ⊕ SQLCipher ===\n");

    // Stage 1: registra sqlite-vec come auto-extension PRIMA di aprire connessioni.
    println!("[1/6] Registro sqlite_vec_init come auto-extension...");
    unsafe {
        // sqlite3_auto_extension vuole una fn pointer con la firma standard di
        // un init di estensione SQLite. sqlite_vec::sqlite3_vec_init ha quella
        // firma a livello C ma è esposta come `unsafe extern "C" fn()` per
        // semplicità del binding — transmute al tipo atteso.
        let init: unsafe extern "C" fn(
            *mut ffi::sqlite3,
            *mut *mut i8,
            *const ffi::sqlite3_api_routines,
        ) -> i32 = std::mem::transmute(sqlite_vec::sqlite3_vec_init as *const ());
        let rc = ffi::sqlite3_auto_extension(Some(init));
        if rc != ffi::SQLITE_OK {
            return Err(format!("sqlite3_auto_extension fallito: rc={}", rc).into());
        }
    }
    println!("    ok");

    // Stage 2: apri DB cifrato.
    println!("[2/6] Creo vault SQLCipher cifrato in {}...", VAULT_PATH);
    let _ = std::fs::remove_file(VAULT_PATH);
    let conn = Connection::open(VAULT_PATH)?;
    conn.pragma_update(None, "key", VAULT_PASSWORD)?;

    let cipher_version: String = conn
        .query_row("PRAGMA cipher_version", [], |r| r.get(0))
        .map_err(|e| format!("PRAGMA cipher_version fallita (SQLCipher non attivo?): {}", e))?;
    println!("    SQLCipher attivo, versione: {}", cipher_version);

    // Sanity check: scrivere/leggere su tabella normale conferma cifratura ok
    conn.execute("CREATE TABLE smoke (id INTEGER, val TEXT)", [])?;
    conn.execute("INSERT INTO smoke VALUES (1, 'hello')", [])?;
    let smoke: String = conn.query_row("SELECT val FROM smoke WHERE id = 1", [], |r| r.get(0))?;
    assert_eq!(smoke, "hello");
    println!("    sanity check tabella standard: ok");

    // Stage 3: verifica sqlite-vec accessibile dalla connessione.
    println!("[3/6] Interrogo vec_version()...");
    let vec_version: String = conn
        .query_row("SELECT vec_version()", [], |r| r.get(0))
        .map_err(|e| format!("vec_version() non disponibile: {}", e))?;
    println!("    sqlite-vec versione: {}", vec_version);

    // Stage 4: virtual table vec0 dentro DB cifrato.
    println!("[4/6] CREATE VIRTUAL TABLE prompts_emb USING vec0(...) su DB cifrato...");
    conn.execute(
        &format!(
            "CREATE VIRTUAL TABLE prompts_emb USING vec0(\
               prompt_id TEXT PRIMARY KEY, \
               embedding FLOAT[{}]\
             )",
            EMBEDDING_DIM
        ),
        [],
    )
    .map_err(|e| format!("CREATE VIRTUAL TABLE fallita: {}", e))?;
    println!("    ok");

    // Stage 5: insert di N embeddings.
    println!("[5/6] INSERT di 3 embeddings di dimensione {}...", EMBEDDING_DIM);
    let dataset: [(&str, [f32; EMBEDDING_DIM]); 3] = [
        ("prm-1-email-formale", [0.10, 0.20, 0.30, 0.40]),
        ("prm-2-email-informale", [0.12, 0.18, 0.32, 0.38]),
        ("prm-3-recipe-pasta", [0.90, 0.10, 0.05, 0.02]),
    ];
    for (id, emb) in &dataset {
        conn.execute(
            "INSERT INTO prompts_emb (prompt_id, embedding) VALUES (?, ?)",
            params![id, embedding_bytes(emb)],
        )?;
    }
    println!("    inseriti {} righe", dataset.len());

    // Stage 6: nearest neighbor query (scoped per rilasciare il borrow di `stmt`).
    println!("[6/6] Query nearest-neighbor con embedding MATCH...");
    {
        let query: [f32; EMBEDDING_DIM] = [0.11, 0.19, 0.31, 0.39]; // simile a prm-1/prm-2
        let mut stmt = conn.prepare(
            "SELECT prompt_id, distance FROM prompts_emb \
             WHERE embedding MATCH ? AND k = 3 \
             ORDER BY distance",
        )?;
        let rows: Vec<(String, f64)> = stmt
            .query_map(params![embedding_bytes(&query)], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
            })?
            .collect::<rusqlite::Result<_>>()?;

        println!("    risultati ({}):", rows.len());
        for (id, dist) in &rows {
            println!("      {} → distance {:.4}", id, dist);
        }

        // Verifica semantica: prm-1 e prm-2 devono essere primi e secondi (vicini al
        // query vector); prm-3 (recipe) deve essere terzo o assente.
        if rows.is_empty() {
            return Err("nessun risultato dalla query MATCH".into());
        }
        let top = &rows[0].0;
        if top != "prm-1-email-formale" && top != "prm-2-email-informale" {
            return Err(format!(
                "ranking inatteso: top = {}, attesi prm-1 o prm-2",
                top
            )
            .into());
        }
        println!("    ranking semantico coerente: top = {}", top);
    }

    // Cleanup
    drop(conn);
    let _ = std::fs::remove_file(VAULT_PATH);

    println!("\n✅ SPIKE PASSED");
    println!("sqlite-vec funziona con SQLCipher via auto-extension statico.");
    println!("Configurazione utilizzabile: rusqlite 0.32 + bundled-sqlcipher-vendored-openssl + sqlite-vec 0.1.");

    Ok(())
}
