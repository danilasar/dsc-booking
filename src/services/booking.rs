use actix_session::Session;
use actix_web::{get, HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use deadpool_postgres::Status;
use serde::Deserialize;
use serde_json::json;
use crate::{AppState, models};
use crate::core::templator;
use crate::models::seat::Seat;
use crate::models::seat::seat_type::SeatType;

#[get("/")]
async fn index(req: HttpRequest, session: Session, app_state: web::Data<AppState<'_>>)
               -> actix_web::Result<HttpResponse>
{
    let service_data = crate::core::ServiceData::new(req, app_state, session).await?;

    let seats = models::seat::get_all_seats(&service_data.client).await;

    let (content, status):(String, StatusCode) = match seats {
        Ok(seats) => {
            let (mut chairs, mut computer_chairs, mut desks, mut poufs)
                :(Vec<Seat>, Vec<Seat>, Vec<Seat>, Vec<Seat>)
                = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for seat in seats {
                let seat_copy = seat.clone();
                match seat.seat_type.unwrap_or(SeatType::Chair) {
                    SeatType::Desk => desks.push(seat_copy),
                    SeatType::Chair => chairs.push(seat_copy),
                    SeatType::ComputerChair => computer_chairs.push(seat_copy),
                    SeatType::Pouf => poufs.push(seat_copy)
                };
            }

            (service_data.app_state.handlebars
            .render("pages/index", &json!({ "seats": {
                "chairs": chairs,
                "computer_chairs": computer_chairs,
                "desks": desks,
                "poufs": poufs
            } }))
            .unwrap_or_default(), StatusCode::OK)},
        Err(e) => (service_data.app_state.handlebars
            .render("errors/seats_unavailable", &json!({ "error": e.to_string() }))
            .unwrap_or_default(), StatusCode::INTERNAL_SERVER_ERROR)
    };

    let wrap = templator::wrap_page(&service_data, &*content, "Главная".into()).await;
    Ok(HttpResponse::build(status)
        .content_type(ContentType::html())
        .body(wrap))
}

#[derive(Deserialize)]
struct SeatPagePath {
    id:i32
}

#[get("/seat/{id}")]
async fn seat_page(req: HttpRequest,
                    session: Session,
                    path: web::Path<SeatPagePath>,
                    app_state: web::Data<AppState<'_>>)
    -> actix_web::Result<HttpResponse>
{
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(path.id.to_string()))
}