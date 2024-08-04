mod config;
mod models;
mod services;
mod core;


use std::{io, sync::Arc};

use actix_files::{Files, NamedFile};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    App, Either,
    error,
    get, http::{
        Method, StatusCode,
    }, HttpRequest, HttpResponse, HttpServer, middleware, Responder, Result, web,
};
use ::config::Config;
use actix_session::config::CookieContentSecurity;
use actix_session::config::SessionLifecycle::BrowserSession;
use actix_web::cookie::{Key, SameSite};
use deadpool_postgres::Pool;
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use tokio_postgres::NoTls;
use crate::config::ServerConfig;

// NOTE: Not a suitable session key for production.
static SESSION_SIGNING_KEY: &[u8] = &[0; 64];

#[derive(Clone)]
struct AppState<'a> {
    handlebars: Arc<Handlebars<'a>>,
    db_pool: Pool
}

/// favicon handler
#[get("/favicon")]
async fn favicon() -> Result<impl Responder> {
    Ok(NamedFile::open("static/favicon.ico")?)
}

fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(
        CookieSessionStore::default(), Key::from(&[0; 64])
    )
        .cookie_secure(false) // https и http
        //.session_lifecycle(BrowserSession::default()) // expire at end of session
        .cookie_same_site(SameSite::Strict)
        .cookie_content_security(CookieContentSecurity::Private) // encrypt
        .cookie_http_only(false) // не отключать чтение скриптами
        .build()
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

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // random key means that restarting server will invalidate existing session cookies
    let key = actix_web::cookie::Key::from(SESSION_SIGNING_KEY);

    dotenv().ok();

    let config_ = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();

    let config: ServerConfig = config_.try_deserialize().unwrap();

    let pool = config.pg.create_pool(None, NoTls).unwrap();

    log::info!("starting HTTP server at http://localhost:8080");

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory("views", DirectorySourceOptions::default())
        .unwrap();


    let state = AppState {
        handlebars: Arc::new(handlebars),
        db_pool: pool
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            // enable automatic response compression - usually register this first
            .wrap(middleware::Compress::default())
            // cookie session middleware
            .wrap(session_middleware())
            // enable logger - always register Actix Web Logger middleware last
            .wrap(middleware::Logger::default())
            // register favicon
            .service(favicon)
            // with path parameters
            .service(web::resource("/user/{name}").route(web::get().to(services::legacy::with_param)))
            // async response body
            .service(web::resource("/async-body/{name}").route(web::get().to(services::legacy::streaming_response)))
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
                        .insert_header((header::LOCATION, "static/welcome.hbs"))
                        .finish()
                })),
            )*/
            .service(services::index::index)
            .service(services::static_pages::about)
            .service(services::users::users)
            .service(services::users::register_get)
            .service(services::users::register_post)
            .service(services::users::login_get)
            .service(services::users::login_post)
            .service(services::users::logout)
            // default
            .default_service(web::to(default_handler))
            .wrap(middleware::NormalizePath::trim())
    })
        .bind(("127.0.0.1", 8080))?
        .workers(2)
        .run()
        .await
}