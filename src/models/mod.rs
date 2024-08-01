use std::future::Future;
use deadpool_postgres::Client;
use tokio_postgres::types::ToSql;
use crate::models::user::User;

pub(crate) mod user;
mod roles;
/*pub async fn select_sql<T>(client: &Client, sql: &str) {
    let mut stmt = String::from(sql);
    stmt = stmt.replace("$table_fields", &T::sql_table_fields());
    //T::jopta();
    let stmt = client.prepare(&stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| T::from_row_ref(row).unwrap())
        .collect::<Vec<User>>();

    Ok(results)
}*/