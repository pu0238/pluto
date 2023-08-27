use std::collections::HashMap;

use pluto::{
    http::{HttpRequest, HttpResponse},
    router::Router,
};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use serde_json::json;
use validator::Validate;

use crate::utils::map_validation_err;

pub(crate) fn setup() -> Router {
    let mut router = Router::new();

    /*  Body validation
     *
     * Path: "/"
     *
     * Body:
     * {
     *  "a": "123",
     *  "b": [1, 2, 3]
     * }
     *
     * For validation we recommend using the library https://crates.io/crates/validator
     */
    router.post("/", false, |req: HttpRequest| async move {
        #[derive(Deserialize, Serialize)]
        struct Body {
            a: Option<String>,
            #[serde(with = "serde_bytes")]
            b: Vec<u8>,
        }

        let my_body: Body = req.body_into_struct()?;

        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": "Hello World from POST",
                "receivedBody": my_body
            })
            .into(),
        })
    });

    /*  Params validation
     *
     * Path: "/19/-1"
     *
     * For validation we recommend using the library https://crates.io/crates/validator
     */
    router.get("/:a/:b/:c", false, |req: HttpRequest| async move {
        #[derive(Deserialize, Serialize, Eq, PartialEq, Debug, Validate)]
        struct Params {
            #[validate(range(min = 18, max = 20))]
            #[serde(deserialize_with = "deserialize_number_from_string")]
            a: u8,
            #[serde(deserialize_with = "deserialize_number_from_string")]
            b: i8,
            #[validate(email)]
            c: String,
        }

        let my_params: Params = req.params_into_struct()?;
        my_params.validate().map_err(map_validation_err)?;

        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": "Hello World from GET",
                "my_params": my_params
            })
            .into(),
        })
    });

    router
}
