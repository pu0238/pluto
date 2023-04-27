use http::{RawHttpRequest, RawHttpResponse, HttpServe};
use std::cell::RefCell;
use ic_cdk_macros::{update, query, post_upgrade};

use crate::{http, router::Router, http_serve, controller};

thread_local! {
    static ROUTER: RefCell<Router>  = RefCell::new(controller::setup());
}

// System functions
#[post_upgrade]
fn pre_upgrade() {
    ROUTER.with(|r| *r.borrow_mut() = controller::setup())
}

// Http interface
#[query]
async fn http_request(req: RawHttpRequest)-> RawHttpResponse {
    let router = ROUTER.with(|r| r.borrow().clone());
    http_serve!(router).serve(req).await
}

#[update]
async fn http_request_update(req: RawHttpRequest)-> RawHttpResponse {
    let router = ROUTER.with(|r| r.borrow().clone());
    http_serve!(router).serve(req).await
}
