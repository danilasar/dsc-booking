use std::collections::HashSet;
use std::hash::{BuildHasher, Hasher};
use std::ops::Deref;
use actix_session::{Session, SessionGetError};
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
use crate::models::user::{add_user, get_user_by_login, get_user_by_token, remove_session_by_token, User, UserLoginForm, UserRegisterForm};
use crate::services::users::AuthError::{BadLogin, BadName, BadPassword};


#[derive(PartialEq)]
enum AuthError {
    BadName,
    BadLogin,
    BadPassword,
    AlreadyExists,
    NotFound,
    TokenNotGenerated,
    CookieNotWrote
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
                       app_state: web::Data<AppState<'_>>, session: Session,
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

        let wrap = templator::wrap_page(&req, &app_state, &session, &client, &register, "Регистрация".into()).await;
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

    let wrap = templator::wrap_page(&req, &app_state, &session, &client, &register, "Регистрация".into()).await;
    Ok(HttpResponse::build(status)
        .content_type(ContentType::html())
        .body(wrap))
}


#[get("/register")]
async fn register_get(req: HttpRequest, session:Session,
                      app_state: web::Data<AppState<'_>>)
    -> actix_web::Result<HttpResponse>
{
    let client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
    let register = app_state.handlebars
        .render("pages/register", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&req, &app_state, &session, &client, &register, "Регистрация".into()).await;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}

async fn generate_login_page(req: HttpRequest,
                       app_state: web::Data<AppState<'_>>, session: Session, client: &Client,
                       user: Option<User>,
                       errors : &Vec<AuthError>)
    -> HttpResponse
{
    let mut data = json!({
                "auth_errors": {
                    "login": errors.contains(&AuthError::BadLogin),
                    "password": errors.contains(&AuthError::BadPassword),
                    "not_found": errors.contains(&AuthError::NotFound),
                    "session": errors.contains(&AuthError::TokenNotGenerated),
                    "cookie": errors.contains(&AuthError::CookieNotWrote)
                }
            });
    if(user.is_some()) {
        data["user"] = json!(user.unwrap());
    }
    let login = app_state.handlebars
        .render("pages/login", &data)
        .unwrap_or_default();

    let wrap = templator::wrap_page(&req, &app_state, &session, client, &login, "Вход".into()).await;
    return HttpResponse::build(StatusCode::BAD_REQUEST)
        .content_type(ContentType::html())
        .body(wrap);
}

#[get("/login")]
async fn login_get(req: HttpRequest, session:Session,
                   app_state: web::Data<AppState<'_>>)
                   -> actix_web::Result<HttpResponse>
{
    let client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
    let login = app_state.handlebars
        .render("pages/login", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&req, &app_state, &session, &client, &login, "Вход".into()).await;
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
    let validation_result = validate_login_form(params.0.clone());
    let mut found = false; // true потому что так надо
    let mut user:User = User {
        id: None,
        login: None,
        name: None,
        password_hash: None,
        role: None,
        score: None,
    };
    if(validation_result.is_ok()) {
        match get_user_by_login(&client, params.login.as_str()).await {
            Ok(usr) => {
                user = usr;
                let hash = hash_password(params.password.as_str(), params.login.as_str());
                found = user.password_hash.clone().unwrap().as_str().eq(&hash);
            },
            Err(..) => found = false
        }
    }
    if(validation_result.is_err() || !found) {
        let mut errors = match validation_result {
            Ok(_) => Vec::new(),
            Err(E) => E
        };
        errors.push(AuthError::NotFound);
        return Ok(generate_login_page(req, app_state, session, &client, Option::from(user), &errors).await);
    }

    let session_token = user::generate_session_token(&client, user.clone(), None).await;

    if(session_token.is_err()) {
        return Ok(generate_login_page(req, app_state, session, &client, Option::from(user), &vec! [AuthError::TokenNotGenerated]).await);
    }

    let session_token = session_token.unwrap();

    match session.insert("token", session_token.key.clone().unwrap()) {
        Ok(_) => Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, "/"))
            .finish()),
        Err(_) => Ok(generate_login_page(req, app_state, session, &client, Option::from(user), &vec! [AuthError::CookieNotWrote]).await)
    }
}

enum GetCurrentUserError {
    SessionGet(SessionGetError), Db(DbError), SessionIsNotString
}

pub async fn get_current_user(client: &Client,
                              session: Session)
    -> Result<User, GetCurrentUserError>
{
    let token : String = match session.get("token") {
        Ok(token_option) => match token_option {
            Some(val) =>  val,
            None => return Err(GetCurrentUserError::SessionIsNotString)
        },
        Err(error) => return Err(GetCurrentUserError::SessionGet(error))
    };

    match get_user_by_token(&client, token.as_str()).await {
        Ok(user) => Ok(user),
        Err(error) => Err(GetCurrentUserError::Db(error))
    }
}

pub async fn is_authored(client: &Client, session: Session) -> bool {
    return get_current_user(&client, session).await.is_ok();
}

#[get("/logout")]
async fn logout(req: HttpRequest, app_state: web::Data<AppState<'_>>, session: Session) -> actix_web::Result<HttpResponse> {
    let client: Client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
    match session.remove("token") {
        Some(token) => {
            remove_session_by_token(&client, token.as_str());
            Ok(HttpResponse::Found()
                .insert_header((header::LOCATION, "/"))
                .finish())
        },
        None => Ok(HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::html())
            .body("Сударь, вы попутали берега"))
    }
}

#[get("/users")]
async fn users(req: HttpRequest, app_state: web::Data<AppState<'_>>, session:Session)
               -> actix_web::Result<HttpResponse>
{
    let client: Client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
    let users = models::user::get_users(&client).await?;
    let users_html = app_state.handlebars
        .render("pages/users", &json!({ "users": users }))
        .unwrap_or_default();
    let wrap = templator::wrap_page(&req, &app_state, &session, &client, &*users_html, "Пользователи".into()).await;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}