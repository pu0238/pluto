use std::collections::HashMap;

use ic_cdk::println;
use pluto::{
    http::{HttpRequest, HttpResponse},
    router::Router,
};
use serde_json::json;
use serde::Deserialize;

pub(crate) fn setup() -> Router {
    let mut router = Router::new();

    router.post("/", false, |req: HttpRequest| async move {
        #[derive(Deserialize, Debug)]
        struct Foo {
            a: Option<String>,
            #[serde(with = "serde_bytes")]
            b: Vec<u8>
        }

        let my_body: Foo = req.validate_body().unwrap();
        println!("{:?}", my_body);

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

    router
}
