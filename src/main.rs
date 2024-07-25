mod templator;

use std::{convert::Infallible, io};
use std::sync::Mutex;

use actix_files::{Files, NamedFile};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    error, get,
    http::{
        header::{self, ContentType},
        Method, StatusCode,
    },
    middleware, web, App, Either, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_lab::extract::Path;
use async_stream::stream;
use serde_json;
use serde_json::json;
use std::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};

// NOTE: Not a suitable session key for production.
static SESSION_SIGNING_KEY: &[u8] = &[0; 64];

#[derive(Clone)]
struct AppState<'a> {
    upon_engine: Arc<upon::Engine<'a>>
}

/// favicon handler
#[get("/favicon")]
async fn favicon() -> Result<impl Responder> {
    Ok(NamedFile::open("static/favicon.ico")?)
}

#[get("/api/test")]
async fn test(req: HttpRequest, session: Session) -> Result<HttpResponse> {
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
async fn welcome(req: HttpRequest, session: Session) -> Result<HttpResponse> {
    println!("{req:?}");

    // session
    let mut counter = 1;
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {count}");
        counter = count + 1;
    }

    // set counter to session
    session.insert("counter", counter)?;

    let mut engine = upon::Engine::new();
    engine.add_template("welcome", include_str!("../style/welcome.html")).unwrap_or_default();
    let result = engine.template("welcome")
        .render(upon::value!{ user: { name: "Ivan Afonichev" }})
        .to_string().unwrap_or_default();

    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(result))
}

#[get("/")]
async fn index(req: HttpRequest, session: Session, app_state: web::Data<AppState<'_>>) -> Result<HttpResponse> {
    let index = app_state.upon_engine.template("index")
        .render(upon::value!{  })
        .to_string().unwrap_or("456".to_string());

    let wrap = templator::wrap_page(req, app_state, &*index, "Главная".into());
     Ok(HttpResponse::build(StatusCode::OK)
         .content_type(ContentType::html())
         .body(wrap))
}

#[get("/about")]
async fn about(req: HttpRequest, session: Session, app_state: web::Data<AppState<'_>>) -> Result<HttpResponse> {
    let about = app_state.upon_engine.template("about")
        .render(upon::value!{  })
        .to_string().unwrap_or_default();

    let wrap = templator::wrap_page(req, app_state, &*about, "О доме".into());
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}


async fn default_handler(req_method: Method) -> Result<impl Responder> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open("static/404.html")?
                .customize()
                .with_status(StatusCode::NOT_FOUND);
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

async fn streaming_response(path: web::Path<String>) -> HttpResponse {
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
async fn with_param(req: HttpRequest, Path((name,)): Path<(String,)>) -> HttpResponse {
    println!("{req:?}");

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(format!("Hello {name}!"))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // random key means that restarting server will invalidate existing session cookies
    let key = actix_web::cookie::Key::from(SESSION_SIGNING_KEY);

    log::info!("starting HTTP server at http://localhost:8080");

    let mut upon_engine = upon::Engine::new();
    upon_engine.add_template("wrap", include_str!("../style/wrap.html")).unwrap_or_default();
    upon_engine.add_template("index", include_str!("../style/pages/index.html")).unwrap_or_default();
    upon_engine.add_template("about", include_str!("../style/pages/about.html")).unwrap_or_default();
    upon_engine.add_template("welcome", include_str!("../style/welcome.html")).unwrap_or_default();


    let state = AppState {
        upon_engine: Arc::new(upon_engine)
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            // enable automatic response compression - usually register this first
            .wrap(middleware::Compress::default())
            // cookie session middleware
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(false)
                    .build(),
            )
            // enable logger - always register Actix Web Logger middleware last
            .wrap(middleware::Logger::default())
            // register favicon
            .service(favicon)
            // register simple route, handle all methods
            .service(welcome)
            .service(test)
            // with path parameters
            .service(web::resource("/user/{name}").route(web::get().to(with_param)))
            // async response body
            .service(web::resource("/async-body/{name}").route(web::get().to(streaming_response)))
            .service(
                web::resource("/test").to(|req: HttpRequest| match *req.method() {
                    Method::GET => HttpResponse::Ok(),
                    Method::POST => HttpResponse::MethodNotAllowed(),
                    _ => HttpResponse::NotFound(),
                }),
            )
            .service(web::resource("/error").to(|| async {
                error::InternalError::new(
                    io::Error::new(io::ErrorKind::Other, "test"),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }))
            // static files
            .service(Files::new("/static", "static").show_files_listing())
            // redirect
            /*.service(
                web::resource("/").route(web::get().to(|req: HttpRequest| async move {
                    println!("{req:?}");
                    HttpResponse::Found()
                        .insert_header((header::LOCATION, "static/welcome.html"))
                        .finish()
                })),
            )*/
            .service(index)
            .service(about)
            // default
            .default_service(web::to(default_handler))
    })
        .bind(("127.0.0.1", 8080))?
        .workers(2)
        .run()
        .await
}