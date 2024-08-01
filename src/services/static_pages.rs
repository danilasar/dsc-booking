use actix_session::Session;
use actix_web::{get, HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use deadpool_postgres::Client;
use serde_json::json;
use crate::{AppState, models};
use crate::core::{db, templator};
use crate::core::errors::DbError;

#[get("/")]
async fn index(req: HttpRequest, session: Session, app_state: web::Data<AppState<'_>>)
    -> actix_web::Result<HttpResponse>
{
    let client: Client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;

    /*let new_user = db::add_user(&client, models::User {
        id: Option::None,
        login: "danilasar".to_string(),
        name: "Данила Григорьев".to_string(),
        password_hash: "123456".to_string(),
        role: 1,
        score: 0
    }).await?;

    let users = db::get_users(&client).await?;*/

    let index = app_state.handlebars
        .render("pages/index", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&req, &app_state, &session, &client, &*index, "Главная".into()).await;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}


#[get("/about")]
async fn about(req: HttpRequest, session: Session, app_state: web::Data<AppState<'_>>)
    -> actix_web::Result<HttpResponse>
{
    let client: Client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;

    let about = app_state.handlebars
        .render("pages/about", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&req, &app_state, &session, &client, &*about, "О доме".into()).await;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}