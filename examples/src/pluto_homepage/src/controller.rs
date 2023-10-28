use std::collections::HashMap;

use pluto::{
    http::{HttpRequest, HttpResponse},
    render_view,
    router::Router,
};
use serde_json::json;

pub(crate) fn setup() -> Router {
    let static_files = crate::compiled::templates::statics::STATICS;

    let mut router = Router::new();

    for file in static_files.iter() {
        let name = file.name;
        router.get(
            &format!("/static/{}", name),
            false,
            |req: HttpRequest| async {
                let bytes = file.content.clone();
                Ok(HttpResponse {
                    status_code: 200,
                    headers: HashMap::new(),
                    body: pluto::http::HttpBody::String(String::from_utf8(bytes.to_vec()).unwrap()),
                })
            },
        );
    }
    router.get("/", false, |req: HttpRequest| async move {
        render_view!(crate::compiled::templates::index_html);
    });

    router
}
