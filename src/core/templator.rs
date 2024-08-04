use std::collections::HashMap;
use actix_session::{Session, SessionGetError};
use actix_web::HttpRequest;
use deadpool_postgres::Client;
use handlebars::Handlebars;
use serde_json::json;
use crate::{AppState, models};
use crate::core::ServiceData;
use crate::models::user::get_user_by_token;

pub(crate) async fn wrap_page(service_data: &ServiceData<'_>,
                        content: &str,
                        title: Option<&str>)
    -> String
{
    let requested_with = match service_data.req.headers().get("X-Requested-With") {
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


    if let Ok(option) = service_data.session.get("token") {
        let option : Option<String> = option;
        if let Some(token) = option {
            if let Ok(user) = get_user_by_token(&service_data.client, token.as_str()).await {
                data["user"] = json!(user);
            }
        }
    };

    let wrap = service_data.app_state.handlebars.render("wrap", &data).unwrap();

    return wrap;
}