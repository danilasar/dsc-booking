use std::convert::Infallible;
use actix_session::Session;
use actix_web::{get, HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web_lab::extract::Path;
use async_stream::stream;
use serde_json::json;

#[get("/api/test")]
async fn test(req: HttpRequest, session: Session) -> actix_web::Result<HttpResponse> {
    let res = json!({
       "foo": "bar"
    });

    let str = res.to_string();

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::json())
        .body(str))
}

/// simple index handler
#[get("/welcome")]
async fn welcome(req: HttpRequest, session: Session) -> actix_web::Result<HttpResponse> {
    println!("{req:?}");

    // session
    let mut counter = 1;
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {count}");
        counter = count + 1;
    }

    // set counter to session
    session.insert("counter", counter)?;

    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body("aboba"))
}



pub(crate) async fn streaming_response(path: web::Path<String>) -> HttpResponse {
    let name = path.into_inner();

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .streaming(stream! {
            yield Ok::<_, Infallible>(web::Bytes::from("Hello "));
            yield Ok::<_, Infallible>(web::Bytes::from(name));
            yield Ok::<_, Infallible>(web::Bytes::from("!"));
        })
}

/// handler with path parameters like `/user/{name}`
pub(crate) async fn with_param(req: HttpRequest, Path((name,)): Path<(String,)>) -> HttpResponse {
    println!("{req:?}");

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(format!("Hello {name}!"))
}