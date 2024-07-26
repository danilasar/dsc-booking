use deadpool_postgres::{Client, GenericClient};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper::tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::types::ToSql;
use crate::core::errors::DbError;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
pub struct User {
    pub id: Option<i32>,
    pub login: Option<String>,
    pub name: Option<String>,
    pub password_hash: Option<String>,
    pub role: Option<i32>,
    pub score: Option<i32>
}

pub async fn get_users(client: &Client) -> Result<Vec<User>, DbError> {
    let stmt = include_str!("sql/get_users.sql");
    let stmt = stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>();

    Ok(results)
}

pub async fn add_user(client: &Client, user_info: User) -> Result<User, DbError> {
    let _stmt = include_str!("sql/add_user.sql");
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let query_params : [&(dyn ToSql + Sync); 4] = [
        &user_info.login,
        &user_info.name,
        &user_info.password_hash,
        &user_info.role
    ];

    let q = client
        .query(
            &stmt,
            &query_params,
        );
    let output = q.await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop();
    output
        .ok_or(DbError::NotFound) // more applicable for SELECTs
}

pub async fn get_user_by_id(client: &Client, id:i32) -> Result<User, DbError>
{
    let stmt = include_str!("sql/get_user_by_id.sql");
    let stmt = stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&stmt).await.unwrap();

    let query_params  : [&(dyn ToSql + Sync); 1] = [&id];

    let query = client.query(&stmt, &query_params);
    let output = query.await?.pop();
    if output.is_none() {
        Err::<User, DbError>(DbError::NotFound);
    }
    Ok(User::from_row_ref(&output.unwrap()).unwrap())
}