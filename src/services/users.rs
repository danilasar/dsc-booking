use actix_session::Session;
use actix_web::{get, HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use deadpool_postgres::Client;
use crate::{AppState, models};
use crate::core::errors::DbError;
use crate::core::templator;

#[get("/users")]
async fn users(req: HttpRequest, session: Session, app_state: web::Data<AppState<'_>>)
               -> actix_web::Result<HttpResponse>
{
    let client: Client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
    let users = models::user::get_users(&client).await?;
    let users_html = app_state.upon_engine.template("users")
        .render(upon::value!{ users: [ match models::user::get_user_by_id(&client, 2).await {
            Ok(T)   => T,
            Err(..) => models::user::get_user_by_id(&client, 1).await?
        } ] })
        .to_string().unwrap_or("Хьюстон, у нас проблемы!".to_string());
    let wrap = templator::wrap_page(req, app_state, &*users_html, "Пользователи".into());
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}