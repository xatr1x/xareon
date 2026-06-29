-- Application settings as a simple key-value store. A KV shape keeps the schema
-- stable as the app grows: a new setting is a new key, not a new column or
-- migration. Values are stored as text; the service maps keys to typed fields.
CREATE TABLE settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
