use actix_session::Session;
use actix_web::{get, HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use serde_json::json;
use crate::AppState;
use crate::core::{self, templator};


#[get("/about")]
async fn about(req: HttpRequest, session: Session, app_state: web::Data<AppState<'_>>)
    -> actix_web::Result<HttpResponse>
{
    let service_data = core::ServiceData::new(req, app_state, session).await?;

    let about = service_data.app_state.handlebars
        .render("pages/about", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&service_data, &*about, "О доме".into()).await;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}