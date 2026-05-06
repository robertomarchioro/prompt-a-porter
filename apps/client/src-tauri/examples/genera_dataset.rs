// Generator dataset realistico — Quality gate Fase 3 Step 10.
//
// Crea un vault SQLite di test (no SQLCipher) con N prompt sintetici,
// tag random + embeddings random L2-normalized in vec0. Riusabile per
// criterion bench (ricerca_ibrida) e test di stress su cartelle.
//
// Uso:
//   cargo run --example genera_dataset --release -- --output /tmp/pap-bench.db --prompts 10000
//   cargo run --example genera_dataset --release             # default 1000 prompt
//
// Note di realismo:
// - I body sono frasi italiane/inglesi miste tipiche di un prompt LLM
// - Lunghezze: 50-500 caratteri (P50 ~150) — distribuzione log-normale
// - 5-50 tag totali, ogni prompt 0-4 tag
// - Embeddings: random gauss + L2-normalize (NON è un embedding semantico
//   reale, ma riproduce le proprietà di shape e norma per il bench di
//   sqlite-vec search_nearest)

use rand::distributions::{Alphanumeric, Distribution, WeightedIndex};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use pap_lib::embeddings_store;

const EMBEDDING_DIM: usize = 384;
const SEED: u64 = 0x70A0_F0E1; // "papoffe1" — riproducibilità
const NUM_TAG: usize = 30;

fn main() {
    let args: Args = parse_args();
    println!(
        "🌱 Genero dataset: {} prompt → {}",
        args.prompts,
        args.output.display()
    );

    if args.output.exists() {
        std::fs::remove_file(&args.output).expect("rimozione db esistente");
    }

    embeddings_store::registra_auto_extension();
    let conn = Connection::open(&args.output).expect("open SQLite");
    pap_lib::migrazione::esegui_migrazioni(&conn).expect("migrazioni");
    pap_lib::libreria::assicura_dati_base(&conn).expect("base data");

    let mut rng = StdRng::seed_from_u64(SEED);

    inserisci_tag(&conn, &mut rng);
    inserisci_prompt(&conn, &mut rng, args.prompts);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NULL", [], |r| r.get(0))
        .unwrap();
    let count_emb: i64 = conn
        .query_row("SELECT COUNT(*) FROM PromptsEmbeddings", [], |r| r.get(0))
        .unwrap();

    println!("✅ Done. {count} prompts, {count_emb} embeddings.");
}

struct Args {
    output: PathBuf,
    prompts: usize,
}

fn parse_args() -> Args {
    let mut args = Args {
        output: PathBuf::from("/tmp/pap-bench.db"),
        prompts: 1000,
    };
    let mut iter = std::env::args().skip(1);
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--output" | "-o" => {
                args.output = PathBuf::from(iter.next().expect("--output value"));
            }
            "--prompts" | "-n" => {
                args.prompts = iter
                    .next()
                    .expect("--prompts value")
                    .parse()
                    .expect("usize");
            }
            "--help" | "-h" => {
                println!("Uso: genera_dataset [--output PATH] [--prompts N]");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Argomento sconosciuto: {a}");
                std::process::exit(2);
            }
        }
    }
    args
}

fn inserisci_tag(conn: &Connection, rng: &mut StdRng) {
    let nomi_tag = [
        "scrittura",
        "marketing",
        "tecnico",
        "email",
        "presentazione",
        "review",
        "code",
        "summary",
        "translate",
        "creative",
        "formale",
        "informale",
        "branding",
        "social",
        "blog",
        "sql",
        "python",
        "rust",
        "claude",
        "gpt",
        "analisi",
        "report",
        "pitch",
        "interview",
        "feedback",
        "ricerca",
        "test",
        "doc",
        "spec",
        "qa",
    ];
    for (i, nome) in nomi_tag.iter().take(NUM_TAG).enumerate() {
        let id = format!("t-{i:03}");
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', ?2, datetime('now'), datetime('now'))",
            params![id, nome],
        )
        .unwrap();
        let emb = embedding_random(rng);
        embeddings_store::upsert_tag_embedding(conn, &id, &emb).unwrap();
    }
}

fn inserisci_prompt(conn: &Connection, rng: &mut StdRng, n: usize) {
    let parole_it = [
        "scrivere", "rivedere", "tradurre", "analizzare", "spiegare", "elenco", "punti",
        "chiave", "contesto", "tono", "formale", "professionale", "email", "report",
        "documento", "codice", "funzione", "test", "pitch", "vendita", "branding",
        "messaggio", "presentazione", "introduzione", "conclusione",
    ];
    let parole_en = [
        "write", "rewrite", "translate", "analyze", "explain", "list", "summary",
        "context", "tone", "code", "function", "test", "pitch", "introduction",
        "conclusion", "feedback", "review", "improve", "concise",
    ];

    let weights = WeightedIndex::new([3, 2]).unwrap(); // 60% IT, 40% EN

    let progress_step = (n / 20).max(1);
    for i in 0..n {
        let id = format!("prm-{i:06}");
        let usa_it = weights.sample(rng) == 0;
        let parole = if usa_it { &parole_it[..] } else { &parole_en[..] };
        let titolo = genera_titolo(rng, parole);
        let body = genera_body(rng, parole);
        let folder_id: Option<String> = None; // root per semplicità

        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, 'private', ?4, 1,
                     datetime('now'), datetime('now'))",
            params![id, titolo, body, folder_id],
        )
        .unwrap();

        // FTS5 trigger fa il mirror automatico se presente; se non c'è,
        // popoliamo a mano. Verifichiamo l'esistenza del table.
        if conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE name='PromptsFts'",
                [],
                |_| Ok(()),
            )
            .is_ok()
        {
            conn.execute(
                "INSERT OR REPLACE INTO PromptsFts (PromptId, Title, Body) VALUES (?1, ?2, ?3)",
                params![id, titolo, body],
            )
            .unwrap();
        }

        // Embedding random L2-normalized (proprietà identica al modello reale).
        let emb = embedding_random(rng);
        embeddings_store::upsert_embedding(conn, &id, &emb).unwrap();

        // 0-4 tag random.
        let n_tag = rng.gen_range(0..=4);
        for _ in 0..n_tag {
            let tag_idx = rng.gen_range(0..NUM_TAG);
            let tag_id = format!("t-{tag_idx:03}");
            // INSERT OR IGNORE per evitare duplicati su stessa coppia (prompt, tag).
            conn.execute(
                "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES (?1, ?2)",
                params![id, tag_id],
            )
            .unwrap();
        }

        if (i + 1) % progress_step == 0 {
            println!("  {} / {n}", i + 1);
        }
    }
}

fn genera_titolo(rng: &mut StdRng, parole: &[&str]) -> String {
    let n = rng.gen_range(2..=5);
    (0..n)
        .map(|_| parole[rng.gen_range(0..parole.len())])
        .collect::<Vec<_>>()
        .join(" ")
}

fn genera_body(rng: &mut StdRng, parole: &[&str]) -> String {
    // Distribuzione log-normale di lunghezza in parole (P50 ~25, max ~80).
    let len_log: f64 = rng.gen_range(2.5..=4.5);
    let n: usize = (len_log.exp() as usize).clamp(8, 80);
    let mut frasi: Vec<String> = Vec::with_capacity(n);
    for _ in 0..n {
        frasi.push(parole[rng.gen_range(0..parole.len())].to_string());
    }
    // Aggiungo qualche segnaposto random per realismo (1 su 5 prompt).
    if rng.gen_bool(0.2) {
        let suffix: String = (0..5).map(|_| rng.sample(Alphanumeric) as char).collect();
        frasi.push(format!("{{{{ {suffix} }}}}"));
    }
    frasi.join(" ")
}

fn embedding_random(rng: &mut StdRng) -> Vec<f32> {
    let mut v: Vec<f32> = (0..EMBEDDING_DIM).map(|_| rng.gen_range(-1.0_f32..1.0)).collect();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-12 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}
