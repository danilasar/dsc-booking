use std::collections::HashMap;
use actix_web::HttpRequest;
use handlebars::Handlebars;
use serde_json::json;
use crate::AppState;

pub(crate) fn wrap_page(req: HttpRequest, app_state: actix_web::web::Data<AppState<'_>>, content: &str, title: Option<&str>) -> String {
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

    let data = json!({ "content": content, "page": { "name": title.unwrap_or_default() } });
    //data.insert("content", content);

    let wrap = app_state.handlebars.render("wrap", &data).unwrap();

    return wrap;
}