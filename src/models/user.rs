use std::hash::{BuildHasher, Hasher};
use chrono::{Duration, Utc};
use deadpool_postgres::{Client, GenericClient};
use rand::Rng;
use rs_sha512::{HasherContext, Sha512State};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper::tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::{Row, Statement};
use tokio_postgres::types::ToSql;
use crate::core::errors::DbError;
use crate::models;

#[derive(Clone, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
pub struct User {
    pub id: Option<i32>,
    pub login: Option<String>,
    pub name: Option<String>,
    pub password_hash: Option<String>,
    pub role: Option<i32>,
    pub score: Option<i32>
}

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "sessions")] // singular 'user' is a keyword..
pub struct Session {
    pub key: Option<String>,
    pub user_id: Option<i32>,
    pub expires: Option<i32> // utc timestamp
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserRegisterForm {
    pub login: String,
    pub name: String,
    pub password: String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserLoginForm {
    pub login: String,
    pub password: String
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
        &user_info.role.unwrap_or(2)
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

async fn get_user_by(client: &Client,
                     sql:&str,
                     query_params: [&(dyn ToSql + Sync); 1])
    -> Result<User, DbError>
{
    let stmt = sql.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&stmt).await.unwrap();
    let query = client.query(&stmt, &query_params);
    let output = query.await?.pop();
    match output {
        Some(T) => Ok(User::from_row_ref(&T).unwrap()),
        None  => Err::<User, DbError>(DbError::from(DbError::NotFound))
    }
}

pub async fn get_user_by_id(client: &Client, id:i32) -> Result<User, DbError>
{
    return get_user_by(client, include_str!("sql/get_user_by_id.sql"), [&id]).await;
}

pub async fn get_user_by_login(client: &Client, login:&str) -> Result<User, DbError> {
    return get_user_by(client,
                       include_str!("sql/get_user_by_login.sql"),
                       [&login]).await;
}

pub async fn get_user_by_token(client: &Client, token: &str) -> Result<User, DbError> {
    return get_user_by(client,
                       include_str!("sql/get_user_by_token.sql"),
                       [&token]).await;
}

pub async fn remove_session_by_token(client: &Client, token: &str) {
    let stmt = include_str!("sql/remove_session_by_token.sql");
    let stmt = client.prepare(stmt).await.unwrap();
    client.query(&stmt, &[&token]);
}

pub async fn remove_user_sessions(client: &Client, user: User) {
    let stmt = include_str!("sql/remove_sessions_by_user.sql");
    let stmt = client.prepare(stmt).await.unwrap();
    client.query(&stmt, &[&user.id]);
}


pub async fn generate_session_token(client: &Client,
                                    mut user: User,
                                    lifetime:Option<Duration>)
                                    -> Result<Session, DbError>
{
    if(user.id.is_none()) {
        if(user.login.is_none()) {
            return Err(DbError::NotFound);
        }
        match get_user_by_login(client, user.login.unwrap().as_str()).await {
            Ok(usr) => user.id = usr.id,
            Err(e) => return Err(e)
        }
    }
    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u128>();
    let mut sha512hasher = Sha512State::default().build_hasher();
    sha512hasher.write(seed.to_string().as_bytes());
    let bytes_result = HasherContext::finish(&mut sha512hasher);
    let token = format!("{bytes_result:02x}");

    let q;

    if(lifetime.is_none()) {
        let _stmt = include_str!("sql/add_session.sql");
        let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
        let stmt = client.prepare(&_stmt).await.unwrap();
        let query_params : [&(dyn ToSql + Sync); 2] = [
            &token,
            &user.id
        ];
        q = client
            .query(
                &stmt,
                &query_params,
            ).await?;
    } else {
        let _stmt = include_str!("sql/add_expirable_session.sql");
        let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
        let stmt = client.prepare(&_stmt).await.unwrap();
        let query_params : [&(dyn ToSql + Sync); 3] = [
            &token,
            &user.id,
            &((Utc::now() + lifetime.unwrap()).timestamp())
        ];
        q = client
            .query(
                &stmt,
                &query_params,
            ).await?;
    }
    let output = q
        .iter()
        .map(|row| Session::from_row_ref(row).unwrap())
        .collect::<Vec<Session>>()
        .pop();
    Ok(Session {
        key: Option::from(token),
        user_id: user.id,
        expires: None,
    })
}