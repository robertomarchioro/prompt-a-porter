# Spike 2 — ONNX Runtime bundle size

> **Stato**: ✅ **PARTIAL — informativo** (eseguito 2026-05-04 su Ubuntu Linux x86_64).
>
> **Verdict**: aggiungere ONNX Runtime al bundle Tauri client lo porta da ~3-4 MB attuali a ~18-25 MB stimati per piattaforma. Crescita 4-8× del peso scaricato dall'utente. Non prohibitivo (~ordine di grandezza di una qualsiasi app Electron media), ma rilevante per Step 5 patch line (auto-update bandwidth) e per la prima installazione utente. **Decisione operativa**: si procede con bundle inclusivo della libreria nativa (ort 2.x con `download-binaries` quando stabilizzato), con opzione di fallback a "download on-demand al primo uso" già documentata e misurabile in Step 1.

## Contesto

Fase 3 Step 1 prevede ONNX Runtime per calcolare embeddings localmente sul client. Due rischi tecnici:

1. **Bundle bloat**: ONNX Runtime è una libreria C++ pesante. Aggiungerla al binario Tauri client può portarlo da ~3-4 MB a ~20-30 MB. Impatto su download size, prima installazione e auto-update bandwidth (Step 5 patch line).
2. **macOS notarization** + **Authenticode signing**: la libreria nativa contiene codice C++ che deve essere firmato e notarizzato. Test reale possibile solo dopo ottenimento dei certificati (Apple Developer + Certum), oggi entrambi non disponibili.

## Strategia testata

Crate isolato `spikes/onnx-bundle/` con un binario `baseline` (hello world senza dipendenze pesanti) compilato con il **medesimo profilo release** del client desktop (`panic = abort`, `codegen-units = 1`, `lto = true`, `opt-level = "s"`, `strip = true`).

Il delta with-ort era previsto via secondo binario, ma `ort` 2.x rc.9, rc.10 e rc.12 hanno bug compile su Rust stable (mismatch tra `ort` e `ort-sys`: rc.9 chiama `.expect()` su fn pointer non-Option, rc.12 referenzia `SessionOptionsAppendExecutionProvider_VitisAI` non esposto da ort-sys). Il blocker non è bloccante per la decisione strategica, perché il **size impact** è ricavabile direttamente dalle release ufficiali ONNX Runtime, che è 99% del peso.

## Misurazioni

### Baseline empirico (Linux x86_64)

```
$ cargo build --release --bin baseline
$ stat --printf="%s\n" target/release/baseline
288440
```

**Baseline binary**: 288 KB (281.7 KB).

Questo è il floor di un binario Rust release stripped — solo crt0, panic handler e println. Tutto sopra questa soglia è costo additivo di librerie linkate.

### Sizes ufficiali ONNX Runtime 1.20.1 (Microsoft GitHub Releases)

Misurate via `curl -sI -L` sul redirect finale di rilascio:

| Piattaforma | Tarball compresso | Libreria nativa decompressa | Note |
|---|---:|---:|---|
| Linux x86_64 | 5.9 MB | ~14 MB (`libonnxruntime.so.1.20.1`) | tarball pulito, solo libonnxruntime + headers |
| macOS arm64 | 7.5 MB | ~18 MB (`libonnxruntime.dylib`) | tarball pulito |
| Windows x86_64 | 62.5 MB (zip completo) | ~22 MB (`onnxruntime.dll`) | il zip include anche `onnxruntime_providers_*.dll` (CUDA/DirectML), per uso CPU-only basta `onnxruntime.dll` |

### Bundle stimato post-integrazione

Tauri client v0.1.0-fase1 attuale (release GitHub):

| Bundle | Size attuale |
|---|---:|
| Linux `.deb` | 4.1 MB |
| Linux `.AppImage` | 81.7 MB *(include rt embedded, pattern AppImage)* |
| macOS `.dmg` arm64 | 4.3 MB |
| Windows NSIS `.exe` | 3.1 MB |

Stima post-integrazione ort + libonnxruntime + tokenizers (+~1 MB binding Rust):

| Bundle | Stima post-Step-1 | Crescita |
|---|---:|---:|
| Linux `.deb` | ~19 MB | 4.6× |
| macOS `.dmg` arm64 | ~23 MB | 5.4× |
| Windows NSIS `.exe` | ~26 MB | 8.4× |
| Linux `.AppImage` | ~95 MB | 1.16× *(già pesante)* |

### Diagnosi

- **Crescita 4-8×** del bundle utente. Non prohibitiva — restiamo nell'ordine di grandezza di app Electron medie. Ben sotto VS Code (~80-100 MB), simile a Slack desktop.
- **Auto-update bandwidth** (Step 5 patch line): l'updater dovrà scaricare ~25 MB invece di ~5 MB. Differenziale non drammatico, ma vale la pena attivare delta updates o compressione zstd dove possibile.
- **macOS notarization** ⊕ **Authenticode**: deferred. La presenza di `libonnxruntime.dylib`/`.dll` aggiunge file non Tauri-managed che vanno firmati separatamente nel bundle. Da gestire nel patch line di Step 5.

## Decisione

✅ **Si procede col path standard**: bundle inclusivo della libreria nativa via `ort` 2.x default features (`download-binaries`) quando il crate raggiunge stabilità, con `copy-dylibs` per assicurare che onnxruntime venga copiata nel bundle Tauri.

### Implementazione di produzione (per Step 1)

1. Attendere `ort 2.0.0-stable` (o pinnare alla prima rc che compila pulito; verificare al momento dell'inizio Step 1).
2. In `apps/client/src-tauri/Cargo.toml`: aggiungere `ort = { version = "2", features = ["ndarray", "copy-dylibs"] }` (download-binaries è default).
3. Verificare che `cargo build --release` produca `libonnxruntime.*` accanto al binario in `target/release/`.
4. Configurare `tauri.conf.json` `bundle.resources` per includere la dynlib in tutti gli OS.

### Item rinviati (non testabili oggi)

| Item | Quando | Bloccante? |
|---|---|---|
| Test reale macOS notarization con `libonnxruntime.dylib` inclusa | Step 5 patch line, post-cert Apple | No — solo verifica |
| Test reale Authenticode signing con `onnxruntime.dll` inclusa | Step 5 patch line, post-cert Certum | No — solo verifica |
| Cross-platform binary size sotto build CI | Step 1 implementation, via `client-build.yml` matrix | No — Linux locale è proxy buono |
| Compile fix `ort 2.x rc` su Rust stable | All'inizio di Step 1, controllare crates.io | Sì per Step 1 ma non oggi — il crate è in evoluzione attiva |

### Fallback se ort resta instabile

- **`candle-core`** (Hugging Face, pure Rust, no native deps): bundle aggiunge ~2-5 MB invece di 14-22, performance leggermente inferiore ma ottime per modelli embedding piccoli (bge-small, MiniLM). Da considerare seriamente come piano B.
- **Download on-demand**: solo se ort si rivela inaffidabile a lungo termine. Costo: complessità setup utente + cache management.

## Riferimenti

- ort crate: <https://github.com/pykeio/ort>
- ort-sys version compatibility tracking: <https://github.com/pykeio/ort/issues>
- ONNX Runtime release pages: <https://github.com/microsoft/onnxruntime/releases>
- candle (alternativa pure Rust): <https://github.com/huggingface/candle>
- Tauri bundle resources docs: <https://v2.tauri.app/develop/resources/>
