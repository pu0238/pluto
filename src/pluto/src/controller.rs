use std::collections::HashMap;

use serde_json::{json};

use crate::{http::{HttpRequest, HttpResponse}, router::Router};

pub fn setup() -> Router{
    let mut router =  Router::new();

    router.get("/", false, |_req: HttpRequest| async move {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": "Hello World",
            }).into()
        })
    });

    router
}