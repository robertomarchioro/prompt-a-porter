# Spikes — materiale esplorativo

Prototipi usa-e-getta creati per validare decisioni tecniche **prima** di implementarle
nel prodotto. Non fanno parte del prodotto, non sono buildati dalla CI e non ricevono
manutenzione: restano nel repo come evidenza empirica delle ADR che li citano.

| Spike | Cosa ha validato | ADR di riferimento |
|---|---|---|
| [`sqlite-vec/`](./sqlite-vec/) | sqlite-vec ⊕ SQLCipher via auto-extension statico (PASSED) | [`sqlite-vec-sqlcipher.md`](../docs/architettura/decisioni/sqlite-vec-sqlcipher.md) |
| [`onnx-bundle/`](./onnx-bundle/) | Impatto di ONNX Runtime sulla dimensione del bundle Tauri | [`onnx-bundle.md`](../docs/architettura/decisioni/onnx-bundle.md) |
| [`embedding-models/`](./embedding-models/) | Scelta del modello di embedding (recall@5 su dataset IT) | [`embedding-model.md`](../docs/architettura/decisioni/embedding-model.md) |
