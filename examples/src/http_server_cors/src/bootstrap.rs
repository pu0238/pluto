use ic_cdk::{post_upgrade, query, update};
use ic_pluto::{
    cors::Cors,
    http::{HttpServe, RawHttpRequest, RawHttpResponse},
    http_serve,
    method::Method,
    router::Router,
};
use std::cell::RefCell;

use crate::controller;

thread_local! {
    static ROUTER: RefCell<Router>  = RefCell::new(controller::setup());
}

// System functions
#[post_upgrade]
fn post_upgrade() {
    ROUTER.with(|r| *r.borrow_mut() = controller::setup())
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
    let cors = Cors::new()
        .allow_origin("*")
        .allow_methods(vec![Method::POST, Method::PUT])
        .allow_headers(vec!["Content-Type", "Authorization"])
        .max_age(Some(3600));

    app.set_router(router);
    app.use_cors(cors);
    app.serve(req).await
}
