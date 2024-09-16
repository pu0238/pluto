use ic_cdk::{post_upgrade, query, update};
use ic_pluto::{
    http::{HttpServe, RawHttpRequest, RawHttpResponse},
    http_serve,
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
    app.set_router(router);
    app.serve(req).await
}
