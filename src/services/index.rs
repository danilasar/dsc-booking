use actix_session::Session;
use actix_web::{get, HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use serde_json::json;
use crate::AppState;
use crate::core::templator;

#[get("/")]
async fn index(req: HttpRequest, session: Session, app_state: web::Data<AppState<'_>>)
               -> actix_web::Result<HttpResponse>
{
    let service_data = crate::core::ServiceData::new(req, app_state, session).await?;

    /*let new_user = db::add_user(&client, models::User {
        id: Option::None,
        login: "danilasar".to_string(),
        name: "Данила Григорьев".to_string(),
        password_hash: "123456".to_string(),
        role: 1,
        score: 0
    }).await?;

    let users = db::get_users(&client).await?;*/

    let index = service_data.app_state.handlebars
        .render("pages/index", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&service_data, &*index, "Главная".into()).await;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}