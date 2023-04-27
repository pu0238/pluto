use candid::{CandidType, Deserialize};
use matchit::{Match, Params};
use serde::Serialize;
use std::{collections::{HashMap}, str::FromStr};
use crate::{method::{Method}, router::{Router, HandlerContainer}};
use serde_json::{json, Value};

#[derive(CandidType, Deserialize, Clone)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize, Clone)]
pub struct RawHttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

impl From<RawHttpRequest> for HttpRequest {
    fn from(req: RawHttpRequest) -> Self {
        HttpRequest {
            method: req.method,
            url: req.url,
            headers: req.headers,
            body: req.body.clone(),
            params: HashMap::default(),
            path: String::default(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
    pub path: String,
}

#[derive(CandidType, Deserialize)]
pub struct RawHttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    pub upgrade: Option<bool>
}

impl RawHttpResponse {
    fn set_upgrade(&mut self, upgrade: bool){
        self.upgrade = Some(upgrade);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HttpBody {
    Value(Value),
    String(String)
}

impl From<HttpBody> for Vec<u8> {
    fn from(b: HttpBody) -> Self {
        return match b.clone() {
            HttpBody::Value(json) =>
                json
                    .to_string()
                    .into_bytes()
                    .into(),
            HttpBody::String(string) =>
                string
                    .into_bytes()
                    .into()
        }
    }
}

impl From<String> for HttpBody {
    fn from(s: String) -> Self {
        HttpBody::String(s)
    }
}

impl From<Value> for HttpBody {
    fn from(j: Value) -> Self {
        HttpBody::Value(j)
    }
}

pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: HttpBody,
}

impl From<HttpResponse> for RawHttpResponse {
    fn from(res: HttpResponse) -> Self {
        let mut headers = res.headers;
        headers.insert(
            String::from("Content-Type"),
            String::from("application/json"));
        RawHttpResponse {
            status_code: res.status_code,
            headers: headers,
            body: res.body.into(),
            upgrade: Some(false)
        }
    }
}

#[macro_export]
macro_rules! http_serve {
    ($arg:expr) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        let name = &name[..name.len() - 3];
        let http_request_type;
        if name.contains("http_request::{{closure}}") {
            http_request_type = "http_request"
        } else if name.contains("http_request_update::{{closure}}") {
            http_request_type = "http_request_update"
        } else {
            panic!("Function \"http_request\" not found")
        }
        HttpServe::new($arg, http_request_type)
    }}
}

pub struct HttpServe {
    router: Router,
    is_query: bool
}

impl HttpServe {
    pub fn new(r: Router, init_name: &str) -> Self {
        let created_in_query = match init_name {
            "http_request_update" => false,
            &_ => true
        };
        Self {
            router: r,
            is_query: created_in_query
        }
    }

    pub fn bad_request_error(error: serde_json::Value) -> Result<(), HttpResponse> {
        return Err(HttpResponse {
            status_code: 400,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 400,
                "message": "Bad Request",
                "error": error
            }).into()
        })
    }

    pub fn internal_server_error() -> Result<(), HttpResponse> {
        return Err(HttpResponse {
            status_code: 500,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 500,
                "message": "Internal server error",
            }).into()
        })
    }

    pub fn not_found_error(message: String) -> Result<(), HttpResponse> {
        return Err(HttpResponse {
            status_code: 404,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 404,
                "message": message,
                "error": "Not Found"
            }).into()
        })
    }

    fn get_path(url: &str) -> &str {
        let mut path = url.split('?').next().unwrap_or("");
        if path.ends_with("/") {
            let mut chars = path.chars();
            chars.next_back();
            path = chars.as_str();
        }
        path
    }

    fn params_to_string(params: Params) -> HashMap<String, String> {
        let mut param: HashMap<String, String> = HashMap::new();
        for val in params.iter() {
            param.insert(String::from(val.0), String::from(val.1));
        }
        param
    }

    async fn build_and_execute_request(
        req: RawHttpRequest,
        path: &str,
        lookup: Match<'_, '_, &HandlerContainer>,
        upgrade: bool
    ) -> RawHttpResponse {
        let mut req: HttpRequest = req.clone().into();
        req.path = String::from(path);
        req.params = Self::params_to_string(lookup.params);
        let handle_res = lookup.value.handler.handle(req).await;
        let mut res: RawHttpResponse = match handle_res {
            Ok(res) => res.into(),
            Err(err_res) => err_res.into(),
        };
        res.set_upgrade(upgrade);
        res
    }

    pub async fn serve (self, req: RawHttpRequest) -> RawHttpResponse{
        match Method::from_str(req.method.clone().as_ref()) {
            Err(_) => Self::internal_server_error().unwrap_err().into(),
            Ok(method) => {
                let path = Self::get_path(req.url.as_ref());
                match self.router.lookup(method, path) {
                    Err(message) => return Self::not_found_error(message).unwrap_err().into(),
                    Ok(lookup) => {
                        let upgrade = lookup.value.upgrade;
                        if self.is_query && upgrade {
                            let mut err: RawHttpResponse = Self::internal_server_error().unwrap_err().into();
                            err.set_upgrade(upgrade);
                            return err
                        }
                        return Self::build_and_execute_request(req.clone(), path, lookup, upgrade).await
                    }
                }
            }
        }
    }
}