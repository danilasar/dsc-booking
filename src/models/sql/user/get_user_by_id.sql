SELECT
    $table_fields
FROM
    public.users
WHERE
    id = $1;