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
    /// I metadati del vault (es. salt hex) sono corrotti o illeggibili.
    /// Causa: decodifica salt fallita in hex_a_bytes. Non dipende dalla password.
    MetadatiDanneggiati(String),
    /// La derivazione della chiave crittografica (Argon2) è fallita
    /// per un problema interno ai parametri. Non dipende dalla password scelta.
    DerivazioneFallita(String),
    /// Il generatore di numeri casuali del sistema operativo non è disponibile.
    /// Motivo opaco: nessun dettaglio interno esposto per evitare leakage.
    /// Semantica fail-closed: l'operazione viene interrotta senza procedere
    /// con un buffer non inizializzato o debole.
    RngNonDisponibile,
    /// Errore di validazione/dominio business (es. nome cartella vuoto,
    /// destinazione non valida, etc.).
    Generico(String),
    /// Fix #456: l'utente ha tentato di salvare una API key di un provider
    /// AI mentre il vault corrente NON è cifrato. Le API key non devono mai
    /// essere persistite in chiaro su disco: l'utente deve prima cifrare
    /// il vault (comando `vault_cifra`).
    VaultNonCifrato,
    /// L'utente ha tentato di cifrare (`vault_cifra`) un vault che è già
    /// cifrato. Non è un errore di dati: è un no-op indicato all'utente.
    VaultGiaCifrato,
}

impl fmt::Display for PapErrore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VaultChiuso => write!(f, "Il vault è chiuso"),
            Self::VaultGiaAperto => write!(f, "Il vault è già aperto"),
            Self::VaultNonEsiste => write!(f, "Il vault non esiste"),
            Self::PasswordErrata => write!(f, "Password errata"),
            Self::PasswordTroppoCorta => write!(f, "La password deve avere almeno 12 caratteri"),
            Self::Db(_) => write!(f, "Errore interno: database non accessibile."),
            Self::Io(_) => write!(f, "Errore interno: impossibile leggere i dati del vault."),
            Self::Json(_) => write!(f, "Errore interno: dati non leggibili."),
            Self::Migrazione(msg) => write!(f, "Errore migrazione: {msg}"),
            Self::MetadatiDanneggiati(_) => write!(
                f,
                "I metadati del vault sono danneggiati. Ripristina da un backup."
            ),
            Self::DerivazioneFallita(_) => write!(
                f,
                "Errore interno: impossibile derivare la chiave crittografica. \
                Questo non dipende dalla password scelta; se il problema persiste, \
                contatta il supporto."
            ),
            Self::RngNonDisponibile => write!(
                f,
                "Errore interno: il generatore di entropia del sistema operativo \
                non è disponibile. Se il problema persiste, riavvia l'applicazione."
            ),
            Self::Generico(msg) => write!(f, "{msg}"),
            Self::VaultNonCifrato => write!(
                f,
                "Il vault non è cifrato: cifra il vault per poter salvare le API key dei provider AI."
            ),
            Self::VaultGiaCifrato => write!(f, "Il vault è già cifrato."),
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
            "La password deve avere almeno 12 caratteri"
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
        // Json(_) ora produce un messaggio opaco senza testo interno della libreria.
        assert_eq!(pap_err.to_string(), "Errore interno: dati non leggibili.");
        assert!(!pap_err.to_string().contains("non_json"));
    }

    /// Verifica che Json produca una stringa opaca in italiano
    /// senza esporre byte offset o testo parsato dalla libreria serde_json.
    #[test]
    fn display_opaco_json() {
        // Crea un errore serde_json con contesto interno (byte offset etc.)
        let json_err = serde_json::from_str::<serde_json::Value>(
            r#"{"chiave": "valore non chiuso"#,
        )
        .unwrap_err();
        let inner_text = json_err.to_string();
        let pap_err = PapErrore::Json(json_err);

        assert_eq!(
            pap_err.to_string(),
            "Errore interno: dati non leggibili."
        );
        // Il testo interno della libreria non deve trapelare
        assert!(!pap_err.to_string().contains(&inner_text));
        // Il messaggio non deve contenere riferimenti a byte/offset
        assert!(!pap_err.to_string().contains("line"));
        assert!(!pap_err.to_string().contains("column"));
    }

    #[test]
    fn display_rng_non_disponibile() {
        let err = PapErrore::RngNonDisponibile;
        let msg = err.to_string();
        // Messaggio opaco: non espone dettagli interni del sistema.
        assert!(
            msg.contains("Errore interno"),
            "Deve iniziare con 'Errore interno': {msg}"
        );
        assert!(
            msg.contains("entropia") || msg.contains("generatore"),
            "Deve descrivere il problema: {msg}"
        );
        // Nessun byte offset, chiave, salt o valore segreto nel messaggio.
        assert!(!msg.contains("salt"), "Non deve menzionare 'salt': {msg}");
        assert!(!msg.contains("key"), "Non deve menzionare 'key': {msg}");
        // Serializza correttamente come stringa JSON opaca.
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.starts_with('"'), "JSON deve essere una stringa: {json}");
    }

    #[test]
    fn migrazione_con_messaggio() {
        let err = PapErrore::Migrazione("V002 fallita".to_string());
        assert!(err.to_string().contains("V002 fallita"));
    }

    /// Verifica che MetadatiDanneggiati produca il messaggio corretto
    /// senza esporre il dettaglio interno della decodifica hex.
    #[test]
    fn display_metadati_danneggiati() {
        let err = PapErrore::MetadatiDanneggiati("hex di lunghezza dispari".to_string());
        assert_eq!(
            err.to_string(),
            "I metadati del vault sono danneggiati. Ripristina da un backup."
        );
        assert!(!err.to_string().contains("hex di lunghezza dispari"));
        assert!(!err.to_string().contains("dispari"));
    }

    /// Verifica che DerivazioneFallita produca un messaggio chiaro e
    /// non esponga dettagli interni della libreria Argon2.
    #[test]
    fn display_derivazione_fallita() {
        let err = PapErrore::DerivazioneFallita("invalid param".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("Errore interno"),
            "Il messaggio deve iniziare con 'Errore interno': {msg}"
        );
        assert!(
            msg.contains("non dipende dalla password"),
            "Il messaggio deve indicare che non dipende dalla password: {msg}"
        );
        assert!(
            msg.contains("contatta il supporto"),
            "Il messaggio deve suggerire di contattare il supporto: {msg}"
        );
        assert!(!msg.contains("invalid param"), "Non deve esporre dettagli interni");
    }

    /// Verifica che Db e Io producano stringhe opache in italiano
    /// senza esporre il testo interno della libreria.
    #[test]
    fn display_opaco_db_io() {
        let db = PapErrore::Db(rusqlite::Error::QueryReturnedNoRows);
        assert_eq!(db.to_string(), "Errore interno: database non accessibile.");
        assert!(!db.to_string().contains("QueryReturnedNoRows"));

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "accesso negato");
        let io = PapErrore::from(io_err);
        assert_eq!(io.to_string(), "Errore interno: impossibile leggere i dati del vault.");
        assert!(!io.to_string().contains("accesso negato"));
    }
}
