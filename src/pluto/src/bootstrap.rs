use http::{RawHttpRequest, RawHttpResponse, HttpServe};
use std::cell::RefCell;
use ic_cdk_macros::{update, query, post_upgrade};

use crate::{http, router::Router, http_serve, controller, cors::Cors, method::Method};

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
    ic_cdk::println!("{}", req.method);
    cors_bootstrap(http_serve!(), req).await
}

#[update]
async fn http_request_update(req: RawHttpRequest) -> RawHttpResponse {
    cors_bootstrap(http_serve!(), req).await
}

async fn bootstrap(mut app: HttpServe, req: RawHttpRequest) -> RawHttpResponse {
    let router = ROUTER.with(|r| r.borrow().clone());
    app.set_router(router);
    app.serve(req).await
}

async fn cors_bootstrap(mut app: HttpServe, req: RawHttpRequest) -> RawHttpResponse {
    let router = ROUTER.with(|r| r.borrow().clone());
    let cors = Cors::new()
        .allow_origin("*")
        .allow_headers(vec!["Content-Type", "Authorization"])
        .max_age(Some(3600));

    app.set_router(router);
    app.use_cors(cors);
    app.serve(req).await
}
