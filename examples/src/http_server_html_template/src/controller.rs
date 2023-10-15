use std::collections::HashMap;

use pluto::{
    http::{HttpRequest, HttpResponse},
    render_view,
    router::Router,
};
use serde_json::json;

pub(crate) fn setup() -> Router {
    let mut router = Router::new();

    router.get("/", false, |req: HttpRequest| async move {
        render_view!(crate::compiled::templates::index_html);
    });

    router
}
