DELETE FROM
    public.sessions
WHERE
    expires < CURRENT_TIMESTAMP;

INSERT INTO
    public.sessions ("key", user_id)
VALUES
    ($1, $2);