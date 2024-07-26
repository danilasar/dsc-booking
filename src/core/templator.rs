use actix_web::HttpRequest;
use crate::AppState;

pub(crate) fn wrap_page(req: HttpRequest, app_state: actix_web::web::Data<AppState<'_>>, content: &str, title: Option<&str>) -> String {
    let requested_with = match req.headers().get("X-Requested-With") {
        Some(T) => { T.to_str().unwrap_or("") },
        None => { "" }
    };
    if(requested_with == "XMLHttpRequest") {
        return content.to_string();
    }
    let wrap = app_state.upon_engine.template("wrap")
        .render(upon::value!{ content: content, page: { name: title.unwrap_or_default() } })
        .to_string().unwrap_or(content.to_string());
    return wrap
}