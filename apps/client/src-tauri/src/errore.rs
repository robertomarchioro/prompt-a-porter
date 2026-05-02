use std::fmt;

/// Errore unificato per tutti i moduli di Prompt a Porter.
/// Serializzabile per essere restituito dai comandi Tauri al frontend.
#[derive(Debug)]
pub enum PapErrore {
    VaultChiuso,
    VaultGiaAperto,
    VaultNonEsiste,
    PasswordErrata,
    PasswordTroppoCorta,
    Db(rusqlite::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Migrazione(String),
    Argon2(String),
}

impl fmt::Display for PapErrore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VaultChiuso => write!(f, "Il vault è chiuso"),
            Self::VaultGiaAperto => write!(f, "Il vault è già aperto"),
            Self::VaultNonEsiste => write!(f, "Il vault non esiste"),
            Self::PasswordErrata => write!(f, "Password errata"),
            Self::PasswordTroppoCorta => write!(f, "La password deve avere almeno 8 caratteri"),
            Self::Db(e) => write!(f, "Errore database: {e}"),
            Self::Io(e) => write!(f, "Errore I/O: {e}"),
            Self::Json(e) => write!(f, "Errore JSON: {e}"),
            Self::Migrazione(msg) => write!(f, "Errore migrazione: {msg}"),
            Self::Argon2(msg) => write!(f, "Errore derivazione chiave: {msg}"),
        }
    }
}

impl std::error::Error for PapErrore {}

impl From<rusqlite::Error> for PapErrore {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e)
    }
}

impl From<std::io::Error> for PapErrore {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for PapErrore {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl serde::Serialize for PapErrore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
