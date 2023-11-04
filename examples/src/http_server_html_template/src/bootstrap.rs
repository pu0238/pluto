use ic_cdk_macros::{post_upgrade, query, update};
use pluto::{
    http::{HttpResponse, HttpServe, RawHttpRequest, RawHttpResponse},
    http_serve,
    router::Router,
};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use crate::controller;

thread_local! {
    static ROUTER: RefCell<Router>  = RefCell::new(controller::setup());
}

fn use_statics(mut router: RefMut<Router>) {
    for file in crate::compiled::templates::statics::STATICS.iter() {
        router.get(&format!("/static/{}", file.name), false, |_req| async {
            let bytes = file.content.clone();
            let mut headers: HashMap<String, String> = HashMap::new();
            headers.insert("Content-Type".to_string(), file.mime.to_string());
            return Ok(HttpResponse {
                status_code: 200,
                headers,
                body: pluto::http::HttpBody::String(String::from_utf8(bytes.to_vec()).unwrap()),
            });
        });
    }
}

// System functions
#[post_upgrade]
fn post_upgrade() {
    ROUTER.with(|r| {
        *r.borrow_mut() = controller::setup();
        use_statics(r.borrow_mut());
    })
}

// Http interface
#[query]
async fn http_request(req: RawHttpRequest) -> RawHttpResponse {
    bootstrap(http_serve!(), req).await
}

#[update]
async fn http_request_update(req: RawHttpRequest) -> RawHttpResponse {
    bootstrap(http_serve!(), req).await
}

async fn bootstrap(mut app: HttpServe, req: RawHttpRequest) -> RawHttpResponse {
    let router = ROUTER.with(|r| r.borrow().clone());
    app.set_router(router);
    app.serve(req).await
}
