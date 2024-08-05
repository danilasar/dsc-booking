use std::hash::{BuildHasher, Hasher};
use chrono::{Duration, Utc};
use deadpool_postgres::Client;
use rand::Rng;
use rs_sha512::{HasherContext, Sha512State};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::{FromTokioPostgresRow, PostgresMapper};
use tokio_postgres::types::ToSql;
use crate::core::errors::DbError;
use crate::models::user::{get_user_by_login, User};

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "sessions")] // singular 'user' is a keyword..
pub struct Session {
    pub key: Option<String>,
    pub user_id: Option<i32>,
    pub expires: Option<i32> // utc timestamp
}

pub async fn remove_session_by_token(client: &Client, token: &str) {
    let stmt = include_str!("sql/user/remove_session_by_token.sql");
    let stmt = client.prepare(stmt).await.unwrap();
    client.query(&stmt, &[&token]);
}

pub async fn remove_user_sessions(client: &Client, user: User) {
    let stmt = include_str!("sql/user/remove_sessions_by_user.sql");
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
        let _stmt = include_str!("sql/user/add_session.sql");
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
        let _stmt = include_str!("sql/user/add_expirable_session.sql");
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