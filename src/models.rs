use serde::{Deserialize, Serialize};
use tokio_pg_mapper::tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
pub struct User {
    pub id: Option<i32>,
    pub login: String,
    pub name: String,
    pub password_hash: String,
    pub role: i32,
    pub score: i32
}