SELECT
    $table_fields
FROM
    public.users AS usrs
RIGHT JOIN
    public.sessions AS sess
WHERE
    sess.user_id = usrs.id AND sess."key" = $1;