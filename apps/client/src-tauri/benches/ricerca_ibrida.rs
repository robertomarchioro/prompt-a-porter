// Bench ricerca_ibrida — Quality gate Fase 3 Step 10.
//
// Misura P50/P95 dei sotto-componenti della ricerca ibrida su un dataset
// realistico generato da `examples/genera_dataset.rs`.
//
// Componenti misurati (in ordine di dipendenza):
//   1. sanitizza_fts — string transform pura
//   2. cerca_lessicale — query FTS5 con LIMIT
//   3. search_nearest (vec0) — KNN brute force su sqlite-vec
//   4. ricerca_completa_simulata — lex + sem + RRF (senza compute_embedding,
//      che richiede modello ONNX scaricato; embedding query random
//      L2-normalized come stand-in)
//
// Soglia di accettazione: P95 ricerca completa < 100ms su 10k prompt.
//
// Uso:
//   cargo bench --bench ricerca_ibrida
//
// I dataset sono creati in /tmp e cancellati post-bench.

use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rusqlite::{params, Connection};

use pap_lib::embeddings_store;

const EMBEDDING_DIM: usize = 384;
const SEED: u64 = 0x70A0_F0E1;

const QUERIES: &[&str] = &[
    "scrivere email formale",
    "review codice python",
    "translate marketing pitch",
    "spiegare concetto tecnico",
    "summary report analisi",
    "branding social media",
    "test improve concise",
    "introduction conclusion feedback",
];

fn db_test(prompts: usize) -> (Connection, PathBuf) {
    let path = std::env::temp_dir().join(format!("pap-bench-{prompts}.db"));
    if path.exists() {
        std::fs::remove_file(&path).expect("cleanup db");
    }
    embeddings_store::registra_auto_extension();
    let conn = Connection::open(&path).expect("open db");
    pap_lib::migrazione::esegui_migrazioni(&conn).expect("migrazioni");
    pap_lib::libreria::assicura_dati_base(&conn).expect("base data");

    populate(&conn, prompts);
    (conn, path)
}

fn populate(conn: &Connection, n: usize) {
    let parole_it = [
        "scrivere",
        "rivedere",
        "tradurre",
        "analizzare",
        "spiegare",
        "elenco",
        "punti",
        "chiave",
        "contesto",
        "tono",
        "formale",
        "professionale",
        "email",
        "report",
        "documento",
        "codice",
        "funzione",
        "test",
        "pitch",
        "vendita",
        "branding",
        "messaggio",
        "presentazione",
        "introduzione",
        "conclusione",
    ];
    let parole_en = [
        "write",
        "rewrite",
        "translate",
        "analyze",
        "explain",
        "list",
        "summary",
        "context",
        "tone",
        "code",
        "function",
        "test",
        "pitch",
        "introduction",
        "conclusion",
        "feedback",
        "review",
        "improve",
        "concise",
    ];
    let weights = WeightedIndex::new([3, 2]).unwrap();
    let mut rng = StdRng::seed_from_u64(SEED);

    for i in 0..n {
        let id = format!("prm-{i:06}");
        let usa_it = weights.sample(&mut rng) == 0;
        let parole = if usa_it { &parole_it[..] } else { &parole_en[..] };
        let titolo = (0..rng.gen_range(2..=5))
            .map(|_| parole[rng.gen_range(0..parole.len())])
            .collect::<Vec<_>>()
            .join(" ");
        let len_words = rng.gen_range(15..=60);
        let body = (0..len_words)
            .map(|_| parole[rng.gen_range(0..parole.len())])
            .collect::<Vec<_>>()
            .join(" ");

        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, 'private', 1,
                     datetime('now'), datetime('now'))",
            params![id, titolo, body],
        )
        .unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO PromptsFts (PromptId, Title, Body) VALUES (?1, ?2, ?3)",
            params![id, titolo, body],
        )
        .unwrap();
        let emb = embedding_random(&mut rng);
        embeddings_store::upsert_embedding(conn, &id, &emb).unwrap();
    }
}

fn embedding_random(rng: &mut StdRng) -> Vec<f32> {
    let mut v: Vec<f32> = (0..EMBEDDING_DIM)
        .map(|_| rng.gen_range(-1.0_f32..1.0))
        .collect();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-12 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}

// ─────────── Benchmarks ───────────

fn bench_lessicale(c: &mut Criterion) {
    let mut group = c.benchmark_group("cerca_lessicale_fts5");
    group.sample_size(50);

    for &n in &[1000usize, 10_000] {
        let (conn, _) = db_test(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            let mut iter = QUERIES.iter().cycle();
            b.iter(|| {
                let q = iter.next().unwrap();
                let _: Vec<String> = pap_lib::ricerca_ibrida::cerca_lessicale(&conn, q, 50)
                    .expect("cerca_lessicale");
            });
        });
    }
    group.finish();
}

fn bench_search_nearest(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_nearest_vec0");
    group.sample_size(50);

    let mut rng = StdRng::seed_from_u64(SEED ^ 0xCAFE);
    for &n in &[1000usize, 10_000] {
        let (conn, _) = db_test(n);
        let query_emb = embedding_random(&mut rng);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let _: Vec<(String, f64)> =
                    embeddings_store::search_nearest(&conn, &query_emb, 50)
                        .expect("search_nearest");
            });
        });
    }
    group.finish();
}

fn bench_ricerca_completa(c: &mut Criterion) {
    // Combina lex + sem + RRF — il path che il command Tauri esegue meno il
    // compute_embedding (richiede modello ONNX scaricato, fuori scope CI).
    let mut group = c.benchmark_group("ricerca_completa_lex_sem_rrf");
    group.sample_size(50);

    let mut rng = StdRng::seed_from_u64(SEED ^ 0xBEEF);
    for &n in &[1000usize, 10_000] {
        let (conn, _) = db_test(n);
        let query_emb = embedding_random(&mut rng);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            let mut iter = QUERIES.iter().cycle();
            b.iter(|| {
                let q = iter.next().unwrap();
                let lex = pap_lib::ricerca_ibrida::cerca_lessicale(&conn, q, 50)
                    .expect("lex");
                let sem: Vec<String> = embeddings_store::search_nearest(&conn, &query_emb, 50)
                    .expect("sem")
                    .into_iter()
                    .map(|(id, _)| id)
                    .collect();
                let _ = pap_lib::ricerca_ibrida::rrf_fuse(&lex, &sem, 0.5, 60.0);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_lessicale, bench_search_nearest, bench_ricerca_completa);
criterion_main!(benches);
