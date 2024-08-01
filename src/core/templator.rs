use std::collections::HashMap;
use actix_session::{Session, SessionGetError};
use actix_web::HttpRequest;
use deadpool_postgres::Client;
use handlebars::Handlebars;
use serde_json::json;
use crate::{AppState, models};
use crate::models::user::get_user_by_token;

pub(crate) async fn wrap_page(req: &HttpRequest,
                        app_state: &actix_web::web::Data<AppState<'_>>,
                        session:&Session,
                        client: &Client,
                        content: &str,
                        title: Option<&str>)
    -> String
{
    let requested_with = match req.headers().get("X-Requested-With") {
        Some(T) => { T.to_str().unwrap_or("") },
        None => { "" }
    };
    if(requested_with == "XMLHttpRequest") {
        return content.to_string();
    }

    /*let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("wrap", include_str!("../views/wrap.hbs"))
        .unwrap();*/

    let mut data = json!({ "content": content, "page": { "name": title.unwrap_or_default() } });


    if let Ok(option) = session.get("token") {
        let option : Option<String> = option;
        if let Some(token) = option {
            if let Ok(user) = get_user_by_token(client, token.as_str()).await {
                data["user"] = json!(user);
            }
        }
    };

    let wrap = app_state.handlebars.render("wrap", &data).unwrap();

    return wrap;
}