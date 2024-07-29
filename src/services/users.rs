use std::collections::HashSet;
use std::hash::{BuildHasher, Hasher};
use std::ops::Deref;
use actix_session::Session;
use actix_web::{get, post, HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use actix_web::http::{header, StatusCode};
use deadpool_postgres::Client;
use serde_json::json;
use regex::Regex;
use rs_sha512::{HasherContext, Sha512State};
use serde::Deserialize;
use crate::{AppState, models};
use crate::core::errors::DbError;
use crate::core::templator;
use crate::models::user;
use crate::models::user::{add_user, get_user_by_login, User, UserLoginForm, UserRegisterForm};
use crate::services::users::AuthError::{BadLogin, BadName, BadPassword};

#[derive(PartialEq)]
enum AuthError {
    BadName,
    BadLogin,
    BadPassword,
    AlreadyExists,
    NotFound
}


async fn validate_register_form(client : &Client, form: UserRegisterForm) -> Result<(), Vec<AuthError>> {
    let regex_name: Regex = Regex::new(r"[А-Яа-я -]+").unwrap();
    let regex_login: Regex = Regex::new(r"[А-Яа-яA-Za-z0-9\-_()*&^%$#@!+=/,.{}\[\]]+").unwrap();
    let mut auth_errors:Vec<AuthError> = Default::default();
    if(form.name.len() > 128 || !regex_name.is_match(form.name.as_str())) {
        auth_errors.push(AuthError::BadName);
    }
    if(form.login.len() > 128 || !regex_login.is_match(form.login.as_str())) {
        auth_errors.push(AuthError::BadLogin);
    }
    if(get_user_by_login(client, form.login.as_str()).await.is_ok()) {
        auth_errors.push(AuthError::AlreadyExists);
    }
    if(form.login.len() > 256 || form.password.len() < 8 || !regex_login.is_match(form.password.as_str())) {
        auth_errors.push(AuthError::BadPassword);
    }
    if(!auth_errors.is_empty()) {
        return Err(auth_errors);
    }
    Ok(())
}

fn validate_login_form(form: UserLoginForm) -> Result<(), Vec<AuthError>> {
    let regex_name: Regex = Regex::new(r"[А-Яа-я -]+").unwrap();
    let regex_login: Regex = Regex::new(r"[А-Яа-яA-Za-z0-9\-_()*&^%$#@!+=/,.{}\[\]]+").unwrap();
    let mut auth_errors:Vec<AuthError> = Default::default();
    if(!regex_login.is_match(form.login.as_str())) {
        auth_errors.push(AuthError::BadLogin);
    }
    if(!regex_login.is_match(form.password.as_str())) {
        auth_errors.push(AuthError::BadPassword);
    }
    if(!auth_errors.is_empty()) {
        return Err(auth_errors);
    }
    Ok(())
}

fn hash_password(password:&str, login: &str) -> String {
    let mut sha512hasher = Sha512State::default().build_hasher();
    sha512hasher.write(password.as_bytes());
    sha512hasher.write(format!("СВО{}aboba_AntiHohol",
                               login.clone()).as_bytes());
    let bytes_result = HasherContext::finish(&mut sha512hasher);
    return format!("{bytes_result:02x}");
}

#[post("/register")]
async fn register_post(req: HttpRequest,
                       app_state: web::Data<AppState<'_>>,
                       params: web::Form<UserRegisterForm>)
    -> actix_web::Result<HttpResponse>
{
    let client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
    let verify_result = validate_register_form(&client, params.0.clone()).await;
    if(verify_result.is_err()) {
        let errors = verify_result.unwrap_err();
        let register = app_state.handlebars
            .render("pages/register", &json!({
                "auth_errors": {
                    "name": errors.contains(&AuthError::BadName),
                    "login": errors.contains(&AuthError::BadLogin),
                    "password": errors.contains(&AuthError::BadPassword),
                    "exists": errors.contains(&AuthError::AlreadyExists)
                },
                "user": params.0
            }))
            .unwrap_or_default();

        let wrap = templator::wrap_page(req, app_state, &register, "Регистрация".into());
        return Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type(ContentType::html())
            .body(wrap));
    }

    let user_data : User = User {
        id: None, role: None, score: None,
        name: Option::from(params.name.clone()),
        login: Option::from(params.login.clone()),
        password_hash: Option::from(hash_password(params.password.as_str(),
                                                  params.login.as_str()))
    };
    //let client:Client = client.await.map_err(DbError::PoolError)?;
    let template:&str;
    let status:StatusCode;
    match add_user(&client, user_data).await {
        Ok(..) => {
            template = "pages/register_success";
            status = StatusCode::OK
        },
        Err(..) => {
            template = "pages/register_failed";
            status = StatusCode::INTERNAL_SERVER_ERROR
        }
    }
    let register = app_state.handlebars
        .render(template, &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(req, app_state, &register, "Регистрация".into());
    Ok(HttpResponse::build(status)
        .content_type(ContentType::html())
        .body(wrap))
}


#[get("/register")]
async fn register_get(req: HttpRequest,
                      app_state: web::Data<AppState<'_>>)
    -> actix_web::Result<HttpResponse>
{
    let register = app_state.handlebars
        .render("pages/register", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(req, app_state, &register, "Регистрация".into());
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}

#[get("/login")]
async fn login_get(req: HttpRequest,
                   app_state: web::Data<AppState<'_>>)
    -> actix_web::Result<HttpResponse>
{
    let login = app_state.handlebars
        .render("pages/login", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(req, app_state, &login, "Вход".into());
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}

#[post("/login")]
async fn login_post(req: HttpRequest, session: Session,
                    app_state: web::Data<AppState<'_>>,
                    params: web::Form<UserLoginForm>)
    -> actix_web::Result<HttpResponse>
{
    let client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
    let verify_result = validate_login_form(params.0.clone());
    let mut found = true; // true потому что так надо
    let user:User;
    if(verify_result.is_ok()) {
        match get_user_by_login(&client, params.login.as_str()).await {
            Ok(usr) => {
                user = usr;
                let hash = hash_password(params.password.as_str(), params.login.as_str());
                found = hash == user.password_hash.unwrap();
            },
            Err(..) => found = false
        }
    }
    if(verify_result.is_err() || !found) {
        let errors = verify_result.unwrap_err();
        let login = app_state.handlebars
            .render("pages/login", &json!({
                "auth_errors": {
                    "login": errors.contains(&AuthError::BadLogin),
                    "password": errors.contains(&AuthError::BadPassword),
                    "not_found": found
                },
                "user": params.0
            }))
            .unwrap_or_default();

        let wrap = templator::wrap_page(req, app_state, &login, "Вход".into());
        return Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type(ContentType::html())
            .body(wrap));
    }

    let session = user::generate_session_token(client, user);

    /*match session.insert("message", model.message.clone()) {
        Ok(_) => HttpResponse::Created().body("Created."),
        Err(_) => HttpResponse::InternalServerError().body("Error.")
    }*/

    Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, "/"))
        .finish())
}



#[get("/users")]
async fn users(req: HttpRequest, app_state: web::Data<AppState<'_>>)
               -> actix_web::Result<HttpResponse>
{
    let client: Client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
    let users = models::user::get_users(&client).await?;
    let users_html = app_state.handlebars
        .render("pages/users", &json!({ "users": [ match models::user::get_user_by_id(&client, 2).await {
            Ok(T)   => T,
            Err(..) => models::user::get_user_by_id(&client, 1).await?
        } ] }))
        .unwrap_or_default();
    let wrap = templator::wrap_page(req, app_state, &*users_html, "Пользователи".into());
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}