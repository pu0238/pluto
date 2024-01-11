use ic_cdk::{post_upgrade, query, update};
use pluto::{
    http::{HttpResponse, HttpServe, RawHttpRequest, RawHttpResponse},
    http_serve,
    router::Router,
};
use std::{cell::RefCell, collections::HashMap};

use crate::controller;

thread_local! {
    static ROUTER: RefCell<Router>  = RefCell::new(controller::setup());
}

// System functions
#[post_upgrade]
fn post_upgrade() {
    ROUTER.with(|r| {
        // Initialize the controller actions
        let mut instance = controller::setup();
        // Inject static files from the 'static' folder
        pluto::use_static_files!(instance);
        // Save the changes
        *r.borrow_mut() = instance;
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
