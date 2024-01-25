use std::collections::HashMap;

use pluto::{http::HttpRequest, render_view, router::Router};

pub(crate) fn setup() -> Router {
    let mut router = Router::new();
    router.get("/test", false, |_req: HttpRequest| async move {
        render_view!(crate::compiled::templates::index_html);
    });
    router.get("/", false, |_req: HttpRequest| async move {
        render_view!(crate::compiled::templates::index_html);
    });

    router
}
