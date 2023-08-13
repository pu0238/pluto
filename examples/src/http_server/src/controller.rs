use std::collections::HashMap;

use ic_cdk::println;
use pluto::{
    http::{HttpRequest, HttpResponse},
    router::Router,
};
use serde_json::json;

pub(crate) fn setup() -> Router {
    let mut router = Router::new();

    router.put("/:value", false, |req: HttpRequest| async move {
        println!("Hello World from PUT {}", req.params.get("value").unwrap());
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": "Hello World from PUT",
            })
            .into(),
        })
    });
    router.post("/", false, |req: HttpRequest| async move {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": "Hello World from POST",
            })
            .into(),
        })
    });
    router.get("/", false, |_req: HttpRequest| async move {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": "Hello World from GET",
            })
            .into(),
        })
    });

    router
}
