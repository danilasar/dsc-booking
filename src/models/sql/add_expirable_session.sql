DELETE FROM
    public.sessions
WHERE
    expires < CURRENT_TIMESTAMP;
INSERT INTO
    public.sessions ("key", user_id, expires)
VALUES
    ($1, $2, $3);