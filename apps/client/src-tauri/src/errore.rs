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
    /// Errore di validazione/dominio business (es. nome cartella vuoto,
    /// destinazione non valida, etc.).
    Generico(String),
}

impl fmt::Display for PapErrore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VaultChiuso => write!(f, "Il vault è chiuso"),
            Self::VaultGiaAperto => write!(f, "Il vault è già aperto"),
            Self::VaultNonEsiste => write!(f, "Il vault non esiste"),
            Self::PasswordErrata => write!(f, "Password errata"),
            Self::PasswordTroppoCorta => write!(f, "La password deve avere almeno 8 caratteri"),
            Self::Db(_) => write!(f, "Errore interno: database non accessibile."),
            Self::Io(_) => write!(f, "Errore interno: impossibile leggere i dati del vault."),
            Self::Json(e) => write!(f, "Errore JSON: {e}"),
            Self::Migrazione(msg) => write!(f, "Errore migrazione: {msg}"),
            Self::Argon2(_) => write!(f, "Errore interno: derivazione chiave fallita."),
            Self::Generico(msg) => write!(f, "{msg}"),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_varianti() {
        assert_eq!(PapErrore::VaultChiuso.to_string(), "Il vault è chiuso");
        assert_eq!(PapErrore::VaultGiaAperto.to_string(), "Il vault è già aperto");
        assert_eq!(PapErrore::VaultNonEsiste.to_string(), "Il vault non esiste");
        assert_eq!(PapErrore::PasswordErrata.to_string(), "Password errata");
        assert_eq!(
            PapErrore::PasswordTroppoCorta.to_string(),
            "La password deve avere almeno 8 caratteri"
        );
    }

    #[test]
    fn serialize_a_stringa_json() {
        let json = serde_json::to_string(&PapErrore::VaultChiuso).unwrap();
        assert_eq!(json, r#""Il vault è chiuso""#);
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file non trovato");
        let pap_err = PapErrore::from(io_err);
        // Io(_) ora produce un messaggio opaco senza testo interno della libreria.
        assert_eq!(
            pap_err.to_string(),
            "Errore interno: impossibile leggere i dati del vault."
        );
        assert!(!pap_err.to_string().contains("file non trovato"));
    }

    #[test]
    fn from_json_error() {
        let json_err = serde_json::from_str::<String>("non_json").unwrap_err();
        let pap_err = PapErrore::from(json_err);
        assert!(pap_err.to_string().starts_with("Errore JSON"));
    }

    #[test]
    fn migrazione_con_messaggio() {
        let err = PapErrore::Migrazione("V002 fallita".to_string());
        assert!(err.to_string().contains("V002 fallita"));
    }

    /// Verifica che Argon2, Db e Io producano stringhe opache in italiano
    /// senza esporre il testo interno della libreria.
    #[test]
    fn display_opaco_argon2_db_io() {
        let argon2 = PapErrore::Argon2("invalid param".to_string());
        assert_eq!(argon2.to_string(), "Errore interno: derivazione chiave fallita.");
        assert!(!argon2.to_string().contains("invalid param"));

        let db = PapErrore::Db(rusqlite::Error::QueryReturnedNoRows);
        assert_eq!(db.to_string(), "Errore interno: database non accessibile.");
        assert!(!db.to_string().contains("QueryReturnedNoRows"));

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "accesso negato");
        let io = PapErrore::from(io_err);
        assert_eq!(io.to_string(), "Errore interno: impossibile leggere i dati del vault.");
        assert!(!io.to_string().contains("accesso negato"));
    }
}
