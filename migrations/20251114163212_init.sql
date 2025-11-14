CREATE TABLE IF NOT EXISTS gateway_id_mapping (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    gateway_id TEXT NOT NULL UNIQUE,
    token TEXT NOT NULL,
    merchant_private_key TEXT NOT NULL
);
