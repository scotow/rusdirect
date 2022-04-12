CREATE TABLE IF NOT EXISTS redirections (
    path TEXT NOT NULL PRIMARY KEY,
    target TEXT NOT NULL,
    expiration INTEGER NOT NULL
);