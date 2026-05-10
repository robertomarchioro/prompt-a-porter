-- V015: Segnaposti globali (issue #159)
--
-- Storage di valori riutilizzabili tra prompt diversi via sintassi
-- {{globale nome}}. Il valore viene aggiornato all'ultima compilazione
-- del segnaposto e ri-proposto come default nelle compilazioni
-- successive (in qualsiasi prompt che lo usa).

CREATE TABLE IF NOT EXISTS GlobalPlaceholders (
    Name      TEXT NOT NULL PRIMARY KEY,
    Value     TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL DEFAULT (datetime('now'))
);
