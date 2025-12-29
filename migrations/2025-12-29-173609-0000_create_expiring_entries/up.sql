-- Your SQL goes here
CREATE TABLE expiring_entries (
    key TEXT PRIMARY KEY NOT NULL,
    expires_at DATETIME NOT NULL
);
