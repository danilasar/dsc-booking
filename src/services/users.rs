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
use crate::core::{ServiceData, templator, errors::DbError};
use crate::models::user;
use crate::models::user::{add_user, get_user_by_login, get_user_by_token,
                          User, UserLoginForm, UserRegisterForm};
use crate::models::session::remove_session_by_token;
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


async fn validate_register_form(service_data: &ServiceData<'_>,
                                form: UserRegisterForm)
    -> Result<(), Vec<AuthError>>
{
    let regex_name: Regex = Regex::new(r"[А-Яа-я -]+").unwrap();
    let regex_login: Regex = Regex::new(r"[А-Яа-яA-Za-z0-9\-_()*&^%$#@!+=/,.{}\[\]]+").unwrap();
    let mut auth_errors:Vec<AuthError> = Default::default();
    if(form.name.len() > 128 || !regex_name.is_match(form.name.as_str())) {
        auth_errors.push(AuthError::BadName);
    }
    if(form.login.len() > 128 || !regex_login.is_match(form.login.as_str())) {
        auth_errors.push(AuthError::BadLogin);
    }
    if(get_user_by_login(&service_data.client, form.login.as_str()).await.is_ok()) {
        auth_errors.push(AuthError::AlreadyExists);
    }
    if(form.login.len() > 256|| form.password.len() < 8 ||
        !regex_login.is_match(form.password.as_str()))
    {
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

#[post("/register")]
async fn register_post(req: HttpRequest,
                       app_state: web::Data<AppState<'_>>, session: Session,
                       params: web::Form<UserRegisterForm>)
    -> actix_web::Result<HttpResponse>
{
    let service_data = crate::core::ServiceData::new(req, app_state, session).await?;
    let verify_result = validate_register_form(&service_data,
                                               params.0.clone()).await;
    if(verify_result.is_err()) {
        let errors = verify_result.unwrap_err();
        let register = service_data.app_state.handlebars
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

        let wrap = templator::wrap_page(&service_data, &register, "Регистрация".into()).await;
        return Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type(ContentType::html())
            .body(wrap));
    }

    let user_data : User = User {
        id: None, role: None, score: None,
        name: Option::from(params.name.clone()),
        login: Option::from(params.login.clone()),
        password_hash: Option::from(crate::core::users::hash_password(params.password.as_str(),
                                                  params.login.as_str()))
    };
    //let client:Client = client.await.map_err(DbError::PoolError)?;
    let template:&str;
    let status:StatusCode;
    match add_user(&service_data.client, user_data).await {
        Ok(..) => {
            template = "pages/register_success";
            status = StatusCode::OK
        },
        Err(..) => {
            template = "pages/register_failed";
            status = StatusCode::INTERNAL_SERVER_ERROR
        }
    }
    let register = service_data.app_state.handlebars
        .render(template, &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&service_data, &register, "Регистрация".into()).await;
    Ok(HttpResponse::build(status)
        .content_type(ContentType::html())
        .body(wrap))
}


#[get("/register")]
async fn register_get(req: HttpRequest, session:Session,
                      app_state: web::Data<AppState<'_>>)
    -> actix_web::Result<HttpResponse>
{
    let service_data = crate::core::ServiceData::new(req, app_state, session).await?;
    let register = service_data.app_state.handlebars
        .render("pages/register", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&service_data, &register, "Регистрация".into()).await;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}

async fn generate_login_page(service_data: &ServiceData<'_>,
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
    let login = service_data.app_state.handlebars
        .render("pages/login", &data)
        .unwrap_or_default();

    let wrap = templator::wrap_page(&service_data, &login, "Вход".into()).await;
    return HttpResponse::build(StatusCode::BAD_REQUEST)
        .content_type(ContentType::html())
        .body(wrap);
}

#[get("/login")]
async fn login_get(req: HttpRequest, session:Session,
                   app_state: web::Data<AppState<'_>>)
                   -> actix_web::Result<HttpResponse>
{
    let service_data = crate::core::ServiceData::new(req, app_state, session).await?;
    let login = service_data.app_state.handlebars
        .render("pages/login", &json!({  }))
        .unwrap_or_default();

    let wrap = templator::wrap_page(&service_data, &login, "Вход".into()).await;
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
    let service_data = ServiceData::new(req, app_state, session).await?;
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
        match get_user_by_login(&service_data.client, params.login.as_str()).await {
            Ok(usr) => {
                user = usr;
                let hash = crate::core::users::hash_password(params.password.as_str(), params.login.as_str());
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
        return Ok(generate_login_page(&service_data, Option::from(user), &errors).await);
    }

    let session_token = models::session::generate_session_token(&service_data.client,
                                                     user.clone(), None).await;

    if(session_token.is_err()) {
        return Ok(generate_login_page(&service_data,
                                      Option::from(user),
                                      &vec! [AuthError::TokenNotGenerated]).await);
    }

    let session_token = session_token.unwrap();

    match service_data.session.insert("token", session_token.key.clone().unwrap()) {
        Ok(_) => Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, "/"))
            .finish()),
        Err(_) => Ok(generate_login_page(&service_data,
                                         Option::from(user),
                                         &vec! [AuthError::CookieNotWrote]).await)
    }
}

#[get("/logout")]
async fn logout(req: HttpRequest,
                app_state: web::Data<AppState<'_>>,
                session: Session)
    -> actix_web::Result<HttpResponse>
{
    let service_data = ServiceData::new(req, app_state, session).await?;
    match service_data.session.remove("token") {
        Some(token) => {
            remove_session_by_token(&service_data.client, token.as_str());
            Ok(HttpResponse::Found()
                .insert_header((header::LOCATION, "/"))
                .finish())
        },
        None => Ok(HttpResponse::build(StatusCode::UNAUTHORIZED)
            .content_type(ContentType::html())
            .body("Сударь, вы попутали берега"))
    }
}

#[get("/users")]
async fn users(req: HttpRequest, app_state: web::Data<AppState<'_>>, session:Session)
               -> actix_web::Result<HttpResponse>
{
    let service_data = ServiceData::new(req, app_state, session).await?;
    let users = user::get_users(&service_data.client).await?;
    let users_html = service_data.app_state.handlebars
        .render("pages/users", &json!({ "users": users }))
        .unwrap_or_default();
    let wrap = templator::wrap_page(&service_data, &*users_html, "Пользователи".into()).await;
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(wrap))
}