INSERT INTO public.users (login, name, password_hash, role)
VALUES ($1, $2, $3, $4)
RETURNING $table_fields;