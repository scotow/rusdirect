--DELETE FROM redirections ORDER BY expiration DESC LIMIT -1 OFFSET ?; // requires SQLITE_ENABLE_UPDATE_DELETE_LIMIT which is not available in sqlx/libsqlite3-sys.
DELETE FROM redirections
WHERE path IN (
    SELECT path
    FROM redirections
    ORDER BY expiration DESC
    LIMIT -1 OFFSET ?
);
