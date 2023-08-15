use std::collections::HashMap;

use pluto::http::HttpResponse;
use serde_json::json;
use validator::ValidationErrors;

pub fn map_validation_err(err: ValidationErrors) -> HttpResponse{
    HttpResponse {
        status_code: 400,
        headers: HashMap::new(),
        body: json!({
            "statusCode": 400,
            "message": err.to_string(),
        })
        .into()
    }
}