SELECT
    $table_fields
FROM
    public.users
WHERE
    login = $1;