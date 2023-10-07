use std::collections::HashMap;

use ic_cdk::println;
use pluto::{
    http::{HttpRequest, HttpResponse, HttpServe},
    router::Router,
};
use serde_json::json;

pub(crate) fn setup() -> Router {
    let mut router = Router::new();

    router.get("/", false, |req: HttpRequest| async move {
        println!("Hello World from PUT {:?}", req.params.get("value"));
        let mut headers = HashMap::new();
        headers.insert(String::from("Content-Type"), String::from("text/html"));
        let mut index: Vec<u8> = vec![];
        templates::index_html(&mut index).unwrap();
        Ok(HttpResponse {
            status_code: 200,
            headers,
            body: pluto::http::HttpBody::String(String::from_utf8(index).unwrap()),
        })
    });

    router
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
