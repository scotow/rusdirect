DELETE FROM redirections
WHERE strftime('%s', 'now') > expiration;