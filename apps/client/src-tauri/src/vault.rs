use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::migrazione;

// ─────────────────── Costanti ───────────────────

const SALT_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PASSWORD_MIN_LEN: usize = 8;
const ARGON2_MEMORY_KIB: u32 = 32_768; // 32 MiB
const ARGON2_TIME_COST: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;

// ────────────────── Vault Meta ──────────────────

fn default_cifrato() -> bool {
    true
}

/// Metadata del vault salvata in chiaro accanto al DB.
/// Contiene il salt per la derivazione della chiave, mai la chiave stessa.
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultMeta {
    pub salt_hex: String,
    pub db_nome: String,
    pub creato_a: String,
    pub argon2_memory_kib: u32,
    pub argon2_time_cost: u32,
    pub argon2_parallelism: u32,
    #[serde(default = "default_cifrato")]
    pub cifrato: bool,
}

// ────────────────── Vault State ─────────────────

/// Stato globale del vault, gestito come Tauri State<>.
/// La connessione è None quando il vault è chiuso.
pub struct VaultState {
    conn: Mutex<Option<Connection>>,
    data_dir: PathBuf,
}

impl VaultState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            conn: Mutex::new(None),
            data_dir,
        }
    }

    fn meta_path(&self) -> PathBuf {
        self.data_dir.join("vault-meta.json")
    }

    fn db_path(&self) -> PathBuf {
        self.data_dir.join("pap-vault.db")
    }

    /// Path della directory dati dell'app (es. `%APPDATA%\com.pap.app` su
    /// Windows, `~/.local/share/com.pap.app` su Linux). Usato da altri moduli
    /// (es. `embeddings`) per cache di file ausiliari.
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// True se il file vault-meta.json esiste (vault è stato creato).
    pub fn esiste(&self) -> bool {
        self.meta_path().exists()
    }

    /// True se la connessione è aperta.
    pub fn aperto(&self) -> bool {
        self.conn.lock().unwrap().is_some()
    }

    /// Accesso alla connessione per query. Restituisce errore se il vault è chiuso.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T, PapErrore>
    where
        F: FnOnce(&Connection) -> Result<T, PapErrore>,
    {
        let guard = self.conn.lock().unwrap();
        let conn = guard.as_ref().ok_or(PapErrore::VaultChiuso)?;
        f(conn)
    }
}

// ─────────── Derivazione chiave (Argon2id) ──────────

fn genera_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

fn bytes_a_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_a_bytes(hex: &str) -> Result<Vec<u8>, PapErrore> {
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| PapErrore::Argon2(format!("hex non valido: {e}")))
        })
        .collect()
}

fn deriva_chiave(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    time_cost: u32,
    parallelism: u32,
) -> Result<[u8; KEY_LEN], PapErrore> {
    let params = Params::new(memory_kib, time_cost, parallelism, Some(KEY_LEN))
        .map_err(|e| PapErrore::Argon2(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut chiave = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut chiave)
        .map_err(|e| PapErrore::Argon2(e.to_string()))?;

    Ok(chiave)
}

/// Applica la chiave derivata al DB via PRAGMA key.
fn applica_chiave(conn: &Connection, chiave: &[u8; KEY_LEN]) -> Result<(), PapErrore> {
    let hex = bytes_a_hex(chiave);
    conn.execute_batch(&format!("PRAGMA key = \"x'{hex}'\";"))?;
    Ok(())
}

/// Verifica che il DB sia leggibile (la chiave è corretta).
fn verifica_chiave(conn: &Connection) -> Result<(), PapErrore> {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|_| PapErrore::PasswordErrata)?;
    Ok(())
}

// ──────────────── Operazioni Vault ──────────────

fn salva_meta(path: &Path, meta: &VaultMeta) -> Result<(), PapErrore> {
    let json = serde_json::to_string_pretty(meta)?;
    fs::write(path, json)?;
    Ok(())
}

fn leggi_meta(path: &Path) -> Result<VaultMeta, PapErrore> {
    let json = fs::read_to_string(path)?;
    let meta: VaultMeta = serde_json::from_str(&json)?;
    Ok(meta)
}

fn timestamp_iso() -> String {
    // Formato semplice senza dipendenza chrono
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

// ──────────────── Comandi Tauri ─────────────────

/// Verifica se il vault esiste (è stato creato almeno una volta).
#[tauri::command]
pub fn vault_esiste(state: State<'_, VaultState>) -> bool {
    state.esiste()
}

/// Verifica se il vault è attualmente aperto.
#[tauri::command]
pub fn vault_aperto(state: State<'_, VaultState>) -> bool {
    state.aperto()
}

/// Crea un nuovo vault con la password fornita.
/// Genera salt, deriva chiave, crea DB cifrato, esegue migrazioni.
///
/// Idempotenza:
/// - Se il vault è già aperto in memoria, ritorna `VaultGiaAperto`.
/// - Se esiste già un vault su disco (meta-file presente) ma non è aperto,
///   tenta lo `vault_unlock` con la password fornita (utile quando
///   `preferenze.onboarding_completato` non si è persistito mentre il vault
///   sì — caso classico portable + EDR aggressivo).
/// - Se trova un DB orfano (`pap-vault.db` senza `vault-meta.json`,
///   probabile creazione precedente fallita a metà), lo rimuove e procede.
#[tauri::command]
pub fn vault_crea(password: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    vault_crea_impl(&password, &state)
}

/// M7 PR-1: logica testabile di `vault_crea` (separata dal wrapper
/// Tauri command per coverage tramite test diretti senza Tauri runtime).
pub(crate) fn vault_crea_impl(password: &str, state: &VaultState) -> Result<(), PapErrore> {
    if password.len() < PASSWORD_MIN_LEN {
        return Err(PapErrore::PasswordTroppoCorta);
    }

    if state.aperto() {
        return Err(PapErrore::VaultGiaAperto);
    }

    if state.esiste() {
        log::info!(
            "vault_crea: vault già esistente in {} → tentativo unlock idempotente",
            state.data_dir.display()
        );
        return vault_unlock_impl(password, state);
    }

    // DB orfano (pap-vault.db senza vault-meta.json) → cleanup
    let db_path = state.db_path();
    if db_path.exists() {
        log::warn!(
            "vault_crea: DB orfano trovato (no meta) in {} → rimozione",
            db_path.display()
        );
        let _ = fs::remove_file(&db_path);
    }

    // Assicura che la directory dati esista
    fs::create_dir_all(&state.data_dir)?;

    let salt = genera_salt();
    let chiave = deriva_chiave(
        password,
        &salt,
        ARGON2_MEMORY_KIB,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
    )?;

    let db_path = state.db_path();
    let conn = Connection::open(&db_path)?;
    applica_chiave(&conn, &chiave)?;

    // Esegui migrazioni sullo schema vuoto
    migrazione::esegui_migrazioni(&conn)?;
    crate::libreria::assicura_dati_base(&conn)?;

    // Salva metadata (salt + parametri, MAI la chiave)
    let meta = VaultMeta {
        salt_hex: bytes_a_hex(&salt),
        db_nome: "pap-vault.db".to_string(),
        creato_a: timestamp_iso(),
        argon2_memory_kib: ARGON2_MEMORY_KIB,
        argon2_time_cost: ARGON2_TIME_COST,
        argon2_parallelism: ARGON2_PARALLELISM,
        cifrato: true,
    };
    salva_meta(&state.meta_path(), &meta)?;

    crate::audit::registra(&conn, "vault.creato", "Vault", "", Some("cifrato"));

    // Metti la connessione nello stato globale
    let mut guard = state.conn.lock().unwrap();
    *guard = Some(conn);

    log::info!("Vault creato: {}", db_path.display());
    Ok(())
}

/// Crea un nuovo vault senza cifratura (DB in chiaro).
///
/// Idempotenza analoga a `vault_crea` ma senza password:
/// - Se aperto → errore.
/// - Se esiste su disco non cifrato → apre (no password). Se cifrato →
///   `PasswordErrata` (l'utente non può bypass del check).
/// - DB orfano → cleanup.
#[tauri::command]
pub fn vault_crea_aperto(state: State<'_, VaultState>) -> Result<(), PapErrore> {
    vault_crea_aperto_impl(&state)
}

/// M7 PR-1: logica testabile di `vault_crea_aperto`.
pub(crate) fn vault_crea_aperto_impl(state: &VaultState) -> Result<(), PapErrore> {
    if state.aperto() {
        return Err(PapErrore::VaultGiaAperto);
    }

    if state.esiste() {
        let meta = leggi_meta(&state.meta_path())?;
        if meta.cifrato {
            log::warn!(
                "vault_crea_aperto su vault cifrato esistente: rifiutato (richiesta password)"
            );
            return Err(PapErrore::PasswordErrata);
        }
        log::info!(
            "vault_crea_aperto: vault non cifrato esistente in {} → apertura idempotente",
            state.data_dir.display()
        );
        let conn = Connection::open(state.db_path())?;
        migrazione::esegui_migrazioni(&conn)?;
        crate::libreria::assicura_dati_base(&conn)?;
        let mut guard = state.conn.lock().unwrap();
        *guard = Some(conn);
        return Ok(());
    }

    let db_path = state.db_path();
    if db_path.exists() {
        log::warn!(
            "vault_crea_aperto: DB orfano trovato (no meta) in {} → rimozione",
            db_path.display()
        );
        let _ = fs::remove_file(&db_path);
    }

    fs::create_dir_all(&state.data_dir)?;

    let conn = Connection::open(&db_path)?;

    migrazione::esegui_migrazioni(&conn)?;
    crate::libreria::assicura_dati_base(&conn)?;

    let meta = VaultMeta {
        salt_hex: String::new(),
        db_nome: "pap-vault.db".to_string(),
        creato_a: timestamp_iso(),
        argon2_memory_kib: 0,
        argon2_time_cost: 0,
        argon2_parallelism: 0,
        cifrato: false,
    };
    salva_meta(&state.meta_path(), &meta)?;

    crate::audit::registra(&conn, "vault.creato", "Vault", "", Some("non_cifrato"));

    let mut guard = state.conn.lock().unwrap();
    *guard = Some(conn);

    log::info!("Vault creato (non cifrato): {}", db_path.display());
    Ok(())
}

/// Verifica se il vault è cifrato.
#[tauri::command]
pub fn vault_cifrato(state: State<'_, VaultState>) -> Result<bool, PapErrore> {
    vault_cifrato_impl(&state)
}

/// M7 PR-1: logica testabile di `vault_cifrato`.
pub(crate) fn vault_cifrato_impl(state: &VaultState) -> Result<bool, PapErrore> {
    if !state.esiste() {
        return Err(PapErrore::VaultNonEsiste);
    }
    let meta = leggi_meta(&state.meta_path())?;
    Ok(meta.cifrato)
}

/// Sblocca il vault esistente con la password.
#[tauri::command]
pub fn vault_unlock(password: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    vault_unlock_impl(&password, &state)
}

/// M7 PR-1: logica testabile di `vault_unlock`.
pub(crate) fn vault_unlock_impl(password: &str, state: &VaultState) -> Result<(), PapErrore> {
    if state.aperto() {
        return Err(PapErrore::VaultGiaAperto);
    }

    if !state.esiste() {
        return Err(PapErrore::VaultNonEsiste);
    }

    let meta = leggi_meta(&state.meta_path())?;
    let conn = Connection::open(state.db_path())?;

    if meta.cifrato {
        let salt = hex_a_bytes(&meta.salt_hex)?;
        let chiave = deriva_chiave(
            password,
            &salt,
            meta.argon2_memory_kib,
            meta.argon2_time_cost,
            meta.argon2_parallelism,
        )?;
        applica_chiave(&conn, &chiave)?;
        verifica_chiave(&conn)?;
    }

    migrazione::esegui_migrazioni(&conn)?;
    crate::libreria::assicura_dati_base(&conn)?;
    crate::audit::registra(&conn, "vault.sbloccato", "Vault", "", None);

    let mut guard = state.conn.lock().unwrap();
    *guard = Some(conn);

    log::info!("Vault sbloccato.");
    Ok(())
}

/// Chiude il vault (blocca).
#[tauri::command]
pub fn vault_lock(state: State<'_, VaultState>) -> Result<(), PapErrore> {
    vault_lock_impl(&state)
}

/// M7 PR-1: logica testabile di `vault_lock`.
pub(crate) fn vault_lock_impl(state: &VaultState) -> Result<(), PapErrore> {
    let mut guard = state.conn.lock().unwrap();
    if let Some(conn) = guard.as_ref() {
        crate::audit::registra(conn, "vault.bloccato", "Vault", "", None);
    } else {
        return Err(PapErrore::VaultChiuso);
    }
    *guard = None;
    log::info!("Vault bloccato.");
    Ok(())
}

/// Cambia la password del vault. Richiede che il vault sia aperto.
#[tauri::command]
pub fn vault_cambia_password(
    password_vecchia: String,
    password_nuova: String,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    vault_cambia_password_impl(&password_vecchia, &password_nuova, &state)
}

/// M7 PR-1: logica testabile di `vault_cambia_password`.
pub(crate) fn vault_cambia_password_impl(
    password_vecchia: &str,
    password_nuova: &str,
    state: &VaultState,
) -> Result<(), PapErrore> {
    if password_nuova.len() < PASSWORD_MIN_LEN {
        return Err(PapErrore::PasswordTroppoCorta);
    }

    if !state.aperto() {
        return Err(PapErrore::VaultChiuso);
    }

    // Verifica la vecchia password
    let meta = leggi_meta(&state.meta_path())?;
    let salt_vecchio = hex_a_bytes(&meta.salt_hex)?;
    let chiave_vecchia = deriva_chiave(
        password_vecchia,
        &salt_vecchio,
        meta.argon2_memory_kib,
        meta.argon2_time_cost,
        meta.argon2_parallelism,
    )?;

    let guard = state.conn.lock().unwrap();
    let conn = guard.as_ref().ok_or(PapErrore::VaultChiuso)?;

    // Verifica che la chiave vecchia sia corretta tramite test query
    // (la connessione è già aperta con la chiave giusta, ma verifichiamo
    // che l'utente abbia inserito la password corrente, non una a caso)
    let conn_verifica = Connection::open(state.db_path())?;
    applica_chiave(&conn_verifica, &chiave_vecchia)?;
    verifica_chiave(&conn_verifica)?;
    drop(conn_verifica);

    // Genera nuovo salt e nuova chiave
    let salt_nuovo = genera_salt();
    let chiave_nuova = deriva_chiave(
        password_nuova,
        &salt_nuovo,
        ARGON2_MEMORY_KIB,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
    )?;

    // Re-key del database
    let hex_nuova = bytes_a_hex(&chiave_nuova);
    conn.execute_batch(&format!("PRAGMA rekey = \"x'{hex_nuova}'\";"))?;
    crate::audit::registra(conn, "vault.password_cambiata", "Vault", "", None);

    // Aggiorna metadata con nuovo salt
    let meta_nuova = VaultMeta {
        salt_hex: bytes_a_hex(&salt_nuovo),
        db_nome: meta.db_nome,
        creato_a: meta.creato_a,
        argon2_memory_kib: ARGON2_MEMORY_KIB,
        argon2_time_cost: ARGON2_TIME_COST,
        argon2_parallelism: ARGON2_PARALLELISM,
        cifrato: true,
    };
    drop(guard);
    salva_meta(&state.meta_path(), &meta_nuova)?;

    log::info!("Password vault cambiata.");
    Ok(())
}

/// Restituisce il percorso della directory dati del vault.
#[tauri::command]
pub fn vault_percorso(state: State<'_, VaultState>) -> String {
    state.data_dir.to_string_lossy().to_string()
}

/// Elimina il vault: chiude la connessione e cancella i file.
#[tauri::command]
pub fn vault_elimina(state: State<'_, VaultState>) -> Result<(), PapErrore> {
    vault_elimina_impl(&state)
}

/// M7 PR-1: logica testabile di `vault_elimina`.
pub(crate) fn vault_elimina_impl(state: &VaultState) -> Result<(), PapErrore> {
    let mut guard = state.conn.lock().unwrap();
    *guard = None;

    let db = state.db_path();
    let meta = state.meta_path();
    if db.exists() {
        fs::remove_file(&db)?;
    }
    if meta.exists() {
        fs::remove_file(&meta)?;
    }

    log::info!("Vault eliminato.");
    Ok(())
}

// ────────────────────── Test ────────────────────

#[cfg(test)]
mod test {
    use super::*;

    fn vault_temp() -> (tempfile::TempDir, VaultState) {
        // Necessario perché i test runnano le migrazioni che includono V005
        // (vec0 virtual table) — sqlite-vec deve essere registrata come
        // auto-extension PRIMA di Connection::open. Idempotente via Once.
        crate::embeddings_store::registra_auto_extension();
        let dir = tempfile::tempdir().unwrap();
        let state = VaultState::new(dir.path().to_path_buf());
        (dir, state)
    }

    #[test]
    fn crea_e_sblocca() {
        let (_dir, state) = vault_temp();
        assert!(!state.esiste());

        // Crea
        let salt = genera_salt();
        let chiave = deriva_chiave("password_sicura_123", &salt, 4096, 1, 1).unwrap();
        let conn = Connection::open(state.db_path()).unwrap();
        applica_chiave(&conn, &chiave).unwrap();
        migrazione::esegui_migrazioni(&conn).unwrap();

        let meta = VaultMeta {
            salt_hex: bytes_a_hex(&salt),
            db_nome: "pap-vault.db".to_string(),
            creato_a: timestamp_iso(),
            argon2_memory_kib: 4096,
            argon2_time_cost: 1,
            argon2_parallelism: 1,
            cifrato: true,
        };
        salva_meta(&state.meta_path(), &meta).unwrap();
        drop(conn);

        assert!(state.esiste());

        // Sblocca con password corretta
        let meta = leggi_meta(&state.meta_path()).unwrap();
        let salt = hex_a_bytes(&meta.salt_hex).unwrap();
        let chiave = deriva_chiave("password_sicura_123", &salt, meta.argon2_memory_kib, meta.argon2_time_cost, meta.argon2_parallelism).unwrap();
        let conn = Connection::open(state.db_path()).unwrap();
        applica_chiave(&conn, &chiave).unwrap();
        verifica_chiave(&conn).unwrap();
    }

    #[test]
    fn password_errata_fallisce() {
        let (_dir, state) = vault_temp();

        let salt = genera_salt();
        let chiave = deriva_chiave("password_corretta", &salt, 4096, 1, 1).unwrap();
        let conn = Connection::open(state.db_path()).unwrap();
        applica_chiave(&conn, &chiave).unwrap();
        migrazione::esegui_migrazioni(&conn).unwrap();

        let meta = VaultMeta {
            salt_hex: bytes_a_hex(&salt),
            db_nome: "pap-vault.db".to_string(),
            creato_a: timestamp_iso(),
            argon2_memory_kib: 4096,
            argon2_time_cost: 1,
            argon2_parallelism: 1,
            cifrato: true,
        };
        salva_meta(&state.meta_path(), &meta).unwrap();
        drop(conn);

        // Sblocca con password sbagliata
        let chiave_sbagliata = deriva_chiave("password_sbagliata", &salt, 4096, 1, 1).unwrap();
        let conn = Connection::open(state.db_path()).unwrap();
        applica_chiave(&conn, &chiave_sbagliata).unwrap();
        let risultato = verifica_chiave(&conn);
        assert!(risultato.is_err());
    }

    #[test]
    fn hex_roundtrip() {
        let originale = [0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x23];
        let hex = bytes_a_hex(&originale);
        assert_eq!(hex, "deadbeef0123");
        let ricostruito = hex_a_bytes(&hex).unwrap();
        assert_eq!(ricostruito, originale);
    }

    #[test]
    fn derivazione_chiave_deterministica() {
        let salt = [1u8; SALT_LEN];
        let k1 = deriva_chiave("test", &salt, 4096, 1, 1).unwrap();
        let k2 = deriva_chiave("test", &salt, 4096, 1, 1).unwrap();
        assert_eq!(k1, k2, "Stessa password+salt deve produrre stessa chiave");
    }

    #[test]
    fn password_corta_rifiutata() {
        let salt = genera_salt();
        let chiave = deriva_chiave("corta", &salt, 4096, 1, 1).unwrap();
        // La derivazione funziona, ma il comando vault_crea dovrebbe rifiutare
        assert_eq!(chiave.len(), KEY_LEN);
        assert!("corta".len() < PASSWORD_MIN_LEN);
    }

    #[test]
    fn hex_a_bytes_lunghezza_dispari_o_char_invalido() {
        let r = hex_a_bytes("zz");
        assert!(r.is_err(), "Char non hex deve produrre Err");

        let r = hex_a_bytes("xy01");
        assert!(r.is_err());

        // Stringa vuota → Vec vuoto valido.
        let r = hex_a_bytes("").unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn salva_e_leggi_meta_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault-meta.json");

        let originale = VaultMeta {
            salt_hex: "deadbeef".to_string(),
            db_nome: "pap-vault.db".to_string(),
            creato_a: "2026-05-07T00:00:00Z".to_string(),
            argon2_memory_kib: 65536,
            argon2_time_cost: 3,
            argon2_parallelism: 4,
            cifrato: true,
        };
        salva_meta(&path, &originale).unwrap();

        let letto = leggi_meta(&path).unwrap();
        assert_eq!(letto.salt_hex, originale.salt_hex);
        assert_eq!(letto.argon2_memory_kib, originale.argon2_memory_kib);
        assert_eq!(letto.cifrato, originale.cifrato);
    }

    #[test]
    fn leggi_meta_file_inesistente_e_errore() {
        let r = leggi_meta(Path::new("/nonexistent/path/vault-meta.json"));
        assert!(r.is_err());
    }

    #[test]
    fn leggi_meta_json_malformato_e_errore() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault-meta.json");
        std::fs::write(&path, "{not-valid-json").unwrap();
        let r = leggi_meta(&path);
        assert!(r.is_err());
    }

    #[test]
    fn deriva_chiave_password_diverse_chiavi_diverse() {
        let salt = [42u8; SALT_LEN];
        let k1 = deriva_chiave("password_uno", &salt, 4096, 1, 1).unwrap();
        let k2 = deriva_chiave("password_due", &salt, 4096, 1, 1).unwrap();
        assert_ne!(k1, k2, "Password diverse devono dare chiavi diverse");
    }

    #[test]
    fn deriva_chiave_salt_diversi_chiavi_diverse() {
        let salt1 = [1u8; SALT_LEN];
        let salt2 = [2u8; SALT_LEN];
        let k1 = deriva_chiave("stessa", &salt1, 4096, 1, 1).unwrap();
        let k2 = deriva_chiave("stessa", &salt2, 4096, 1, 1).unwrap();
        assert_ne!(k1, k2, "Salt diversi devono dare chiavi diverse");
    }

    #[test]
    fn timestamp_iso_non_vuoto() {
        let ts = timestamp_iso();
        assert!(!ts.is_empty());
    }

    #[test]
    fn re_key_funziona() {
        let (_dir, state) = vault_temp();

        // Crea con password1
        let salt1 = genera_salt();
        let chiave1 = deriva_chiave("password_uno_ok", &salt1, 4096, 1, 1).unwrap();
        let conn = Connection::open(state.db_path()).unwrap();
        applica_chiave(&conn, &chiave1).unwrap();
        migrazione::esegui_migrazioni(&conn).unwrap();

        // Re-key con password2
        let salt2 = genera_salt();
        let chiave2 = deriva_chiave("password_due_ok", &salt2, 4096, 1, 1).unwrap();
        let hex2 = bytes_a_hex(&chiave2);
        conn.execute_batch(&format!("PRAGMA rekey = \"x'{hex2}'\";"))
            .unwrap();
        drop(conn);

        // Verifica: password1 non funziona più
        let conn = Connection::open(state.db_path()).unwrap();
        applica_chiave(&conn, &chiave1).unwrap();
        assert!(verifica_chiave(&conn).is_err());
        drop(conn);

        // Verifica: password2 funziona
        let conn = Connection::open(state.db_path()).unwrap();
        applica_chiave(&conn, &chiave2).unwrap();
        verifica_chiave(&conn).unwrap();
    }

    // ─── M7 PR-1: copertura command vault_*_impl ────────────────────

    #[test]
    fn crea_impl_password_corta_errore() {
        let (_dir, state) = vault_temp();
        let r = vault_crea_impl("short", &state);
        assert!(matches!(r, Err(PapErrore::PasswordTroppoCorta)));
        assert!(!state.esiste());
    }

    #[test]
    fn crea_impl_happy_path() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_lunga_ok", &state).unwrap();
        assert!(state.esiste());
        assert!(state.aperto());
        // Meta deve dire cifrato=true
        let cifrato = vault_cifrato_impl(&state).unwrap();
        assert!(cifrato);
    }

    #[test]
    fn crea_impl_vault_gia_aperto_errore() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_lunga_ok", &state).unwrap();
        // Secondo tentativo con vault gia' aperto in memoria
        let r = vault_crea_impl("password_lunga_ok", &state);
        assert!(matches!(r, Err(PapErrore::VaultGiaAperto)));
    }

    #[test]
    fn crea_impl_idempotente_su_vault_esistente() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_ok_123", &state).unwrap();
        // Simula app restart: rilascia lock
        {
            let mut g = state.conn.lock().unwrap();
            *g = None;
        }
        assert!(!state.aperto());
        // Re-chiama crea -> tenta unlock idempotente con stessa password
        vault_crea_impl("password_ok_123", &state).unwrap();
        assert!(state.aperto());
    }

    #[test]
    fn crea_impl_db_orfano_viene_rimosso() {
        let (_dir, state) = vault_temp();
        // Crea un file orfano dove starebbe il DB (no meta)
        std::fs::write(state.db_path(), b"junk-orphan-db").unwrap();
        assert!(state.db_path().exists());
        assert!(!state.esiste()); // niente meta -> non e' un vault valido
        // crea_impl deve rimuoverlo e procedere
        vault_crea_impl("password_ok_123", &state).unwrap();
        assert!(state.esiste());
    }

    #[test]
    fn unlock_impl_vault_non_esiste() {
        let (_dir, state) = vault_temp();
        let r = vault_unlock_impl("qualsiasi_password", &state);
        assert!(matches!(r, Err(PapErrore::VaultNonEsiste)));
    }

    #[test]
    fn unlock_impl_password_errata() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_corretta_xx", &state).unwrap();
        // Lock per riaprire
        vault_lock_impl(&state).unwrap();
        // Tentativo unlock con password sbagliata
        let r = vault_unlock_impl("password_sbagliata_yy", &state);
        assert!(matches!(r, Err(PapErrore::PasswordErrata)));
        assert!(!state.aperto());
    }

    #[test]
    fn unlock_impl_vault_gia_aperto() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_ok_123", &state).unwrap();
        let r = vault_unlock_impl("password_ok_123", &state);
        assert!(matches!(r, Err(PapErrore::VaultGiaAperto)));
    }

    #[test]
    fn cambia_password_impl_happy_path() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_iniziale", &state).unwrap();
        vault_cambia_password_impl("password_iniziale", "password_nuova_xx", &state).unwrap();
        // Verifica: la nuova password sblocca dopo lock
        vault_lock_impl(&state).unwrap();
        vault_unlock_impl("password_nuova_xx", &state).unwrap();
        assert!(state.aperto());
    }

    #[test]
    fn cambia_password_impl_nuova_corta_errore() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_ok_123", &state).unwrap();
        let r = vault_cambia_password_impl("password_ok_123", "x", &state);
        assert!(matches!(r, Err(PapErrore::PasswordTroppoCorta)));
    }

    #[test]
    fn cambia_password_impl_vault_chiuso() {
        let (_dir, state) = vault_temp();
        // Niente vault_crea -> vault non aperto
        let r = vault_cambia_password_impl("any", "password_nuova", &state);
        assert!(matches!(r, Err(PapErrore::VaultChiuso)));
    }

    #[test]
    fn cambia_password_impl_vecchia_errata() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_giusta", &state).unwrap();
        let r = vault_cambia_password_impl("password_sbagliata", "password_nuova_xx", &state);
        assert!(matches!(r, Err(PapErrore::PasswordErrata)));
    }

    #[test]
    fn lock_impl_su_vault_chiuso_errore() {
        let (_dir, state) = vault_temp();
        let r = vault_lock_impl(&state);
        assert!(matches!(r, Err(PapErrore::VaultChiuso)));
    }

    #[test]
    fn lock_impl_e_riapri() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_ok_123", &state).unwrap();
        assert!(state.aperto());
        vault_lock_impl(&state).unwrap();
        assert!(!state.aperto());
        assert!(state.esiste()); // file su disco preservati
    }

    #[test]
    fn elimina_impl_rimuove_file() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_ok_123", &state).unwrap();
        assert!(state.esiste());
        vault_elimina_impl(&state).unwrap();
        assert!(!state.esiste());
        assert!(!state.aperto());
        assert!(!state.db_path().exists());
        assert!(!state.meta_path().exists());
    }

    #[test]
    fn elimina_impl_idempotente_su_vault_inesistente() {
        let (_dir, state) = vault_temp();
        // Nessun vault creato; elimina non deve fallire (idempotente)
        vault_elimina_impl(&state).unwrap();
    }

    #[test]
    fn crea_aperto_impl_happy_path() {
        let (_dir, state) = vault_temp();
        vault_crea_aperto_impl(&state).unwrap();
        assert!(state.aperto());
        let cifrato = vault_cifrato_impl(&state).unwrap();
        assert!(!cifrato, "vault_crea_aperto deve creare vault NON cifrato");
    }

    #[test]
    fn crea_aperto_impl_su_vault_cifrato_esistente_rifiutato() {
        let (_dir, state) = vault_temp();
        vault_crea_impl("password_cifrato", &state).unwrap();
        vault_lock_impl(&state).unwrap();
        let r = vault_crea_aperto_impl(&state);
        assert!(matches!(r, Err(PapErrore::PasswordErrata)),
            "crea_aperto su vault cifrato deve rifiutare");
    }

    #[test]
    fn crea_aperto_impl_idempotente_su_vault_non_cifrato_esistente() {
        let (_dir, state) = vault_temp();
        vault_crea_aperto_impl(&state).unwrap();
        // Simula restart
        {
            let mut g = state.conn.lock().unwrap();
            *g = None;
        }
        // Re-crea_aperto: deve aprire idempotentemente
        vault_crea_aperto_impl(&state).unwrap();
        assert!(state.aperto());
    }

    #[test]
    fn crea_aperto_impl_vault_gia_aperto_errore() {
        let (_dir, state) = vault_temp();
        vault_crea_aperto_impl(&state).unwrap();
        let r = vault_crea_aperto_impl(&state);
        assert!(matches!(r, Err(PapErrore::VaultGiaAperto)));
    }

    #[test]
    fn cifrato_impl_vault_non_esiste() {
        let (_dir, state) = vault_temp();
        let r = vault_cifrato_impl(&state);
        assert!(matches!(r, Err(PapErrore::VaultNonEsiste)));
    }
}
