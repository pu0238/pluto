use crate::{
    cors::Cors,
    method::Method,
    router::{Handler, HandlerContainer, Router},
};
use candid::{CandidType, Deserialize};
use matchit::{Match, Params};
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, str::FromStr};

#[derive(CandidType, Deserialize, Clone)]
pub(crate) struct HeaderField(pub(crate) String, pub(crate) String);

#[derive(CandidType, Deserialize, Clone)]
pub struct RawHttpRequest {
    pub(crate) method: String,
    pub(crate) url: String,
    pub(crate) headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub(crate) body: Vec<u8>,
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
    pub(crate) method: String,
    pub(crate) url: String,
    pub(crate) headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub(crate) body: Vec<u8>,
    pub params: HashMap<String, String>,
    pub(crate) path: String,
}

impl HttpRequest {
    pub fn validate_body<T: for<'a> Deserialize<'a>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}

#[derive(CandidType, Deserialize)]
pub struct RawHttpResponse {
    pub(crate) status_code: u16,
    pub(crate) headers: HashMap<String, String>,
    #[serde(with = "serde_bytes")]
    pub(crate) body: Vec<u8>,
    pub(crate) upgrade: Option<bool>,
}

impl RawHttpResponse {
    fn set_upgrade(&mut self, upgrade: bool) {
        self.upgrade = Some(upgrade);
    }

    fn enrich_header(&mut self) {
        if let None = self.headers.get("Content-Type") {
            self.headers.insert(
                String::from("Content-Type"),
                String::from("application/json"),
            );
        }
        self.headers
            .insert(String::from("X-Powered-By"), String::from("Pluto"));
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HttpBody {
    Value(Value),
    String(String),
}

impl From<HttpBody> for Vec<u8> {
    fn from(b: HttpBody) -> Self {
        return match b.clone() {
            HttpBody::Value(json) => json.to_string().into_bytes().into(),
            HttpBody::String(string) => string.into_bytes().into(),
        };
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

impl HttpResponse {
    pub fn add_raw_header(&mut self, key: &str, value: String) {
        self.headers.insert(key.to_string(), value);
    }

    pub fn remove_header(&mut self, key: &str) {
        self.headers.remove(key);
    }
}

impl From<HttpResponse> for RawHttpResponse {
    fn from(res: HttpResponse) -> Self {
        let mut res = RawHttpResponse {
            status_code: res.status_code,
            headers: res.headers,
            body: res.body.into(),
            upgrade: Some(false),
        };
        res.enrich_header();
        res
    }
}

#[macro_export]
macro_rules! http_serve_router {
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
        HttpServe::new_with_router($arg, http_request_type)
    }};
}

#[macro_export]
macro_rules! http_serve {
    () => {{
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
        HttpServe::new(http_request_type)
    }};
}

pub struct HttpServe {
    router: Router,
    cors_policy: Option<Cors>,
    is_query: bool,
}

impl HttpServe {
    pub fn new(init_name: &str) -> Self {
        let created_in_query = match init_name {
            "http_request_update" => false,
            &_ => true,
        };
        Self {
            router: Router::new(),
            cors_policy: None,
            is_query: created_in_query,
        }
    }

    pub fn new_with_router(r: Router, init_name: &str) -> Self {
        let created_in_query = match init_name {
            "http_request_update" => false,
            &_ => true,
        };
        Self {
            router: r,
            cors_policy: None,
            is_query: created_in_query,
        }
    }

    pub fn set_router(&mut self, r: Router) {
        self.router = r;
    }

    pub fn bad_request_error(error: serde_json::Value) -> Result<(), HttpResponse> {
        return Err(HttpResponse {
            status_code: 400,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 400,
                "message": "Bad Request",
                "error": error
            })
            .into(),
        });
    }

    pub fn internal_server_error() -> Result<(), HttpResponse> {
        return Err(HttpResponse {
            status_code: 500,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 500,
                "message": "Internal server error",
            })
            .into(),
        });
    }

    pub fn not_found_error(message: String) -> Result<(), HttpResponse> {
        return Err(HttpResponse {
            status_code: 404,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 404,
                "message": message,
                "error": "Not Found"
            })
            .into(),
        });
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
        self,
        req: RawHttpRequest,
        path: &str,
        lookup: Match<'_, '_, &HandlerContainer>,
        upgrade: bool,
    ) -> RawHttpResponse {
        let mut req: HttpRequest = req.clone().into();
        req.path = String::from(path);
        req.params = Self::params_to_string(lookup.params);
        let handle_res = lookup.value.handler.handle(req).await;
        let mut res = Self::unwrap_response(handle_res);
        self.use_plugins(&mut res);
        let mut raw_res: RawHttpResponse = res.into();
        raw_res.set_upgrade(upgrade);
        raw_res
    }

    fn unwrap_response(res: Result<HttpResponse, HttpResponse>) -> HttpResponse {
        match res {
            Ok(res) => res,
            Err(err_res) => err_res,
        }
    }

    fn use_plugins(self, res: &mut HttpResponse) {
        self.add_cors_to_res(res);
    }

    fn add_cors_to_res(self, res: &mut HttpResponse) {
        if let Some(cors) = self.cors_policy {
            cors.merge(res)
        }
    }

    pub fn use_cors(&mut self, cors_policy: Cors) {
        self.cors_policy = Some(cors_policy);
    }

    pub async fn serve(self, req: RawHttpRequest) -> RawHttpResponse {
        match Method::from_str(req.method.clone().as_ref()) {
            Err(_) => Self::internal_server_error().unwrap_err().into(),
            Ok(method) => {
                let path = Self::get_path(req.url.as_ref());
                match self.router.clone().lookup(method, path) {
                    Err(message) => {
                        // Handle OPTIONS request
                        if req.method == Method::OPTIONS.to_string() && self.router.handle_options {
                            let router_clone = self.router.clone();
                            let allow = router_clone.allowed(path);

                            if !allow.is_empty() {
                                return match self.router.global_options {
                                    Some(ref handler) => {
                                        let handle_res = handler.handler.handle(req.into()).await;
                                        let mut raw_res: RawHttpResponse =
                                            Self::unwrap_response(handle_res).into();
                                        raw_res.set_upgrade(handler.upgrade);
                                        raw_res
                                    }
                                    None => {
                                        let mut res = HttpResponse {
                                            status_code: 204,
                                            headers: HashMap::new(),
                                            body: "".to_string().into(),
                                        };
                                        self.use_plugins(&mut res);
                                        if let None =
                                            res.headers.get("Access-Control-Allow-Methods")
                                        {
                                            res.headers.insert(
                                                "Access-Control-Allow-Methods".to_string(),
                                                allow.join(","),
                                            );
                                        }

                                        return res.into();
                                    }
                                };
                            }
                        }

                        return Self::not_found_error(message).unwrap_err().into();
                    }
                    Ok(lookup) => {
                        let upgrade = lookup.value.upgrade;
                        if self.is_query && upgrade {
                            let mut err: RawHttpResponse =
                                Self::internal_server_error().unwrap_err().into();
                            err.set_upgrade(upgrade);
                            return err;
                        }
                        let res = self
                            .build_and_execute_request(req.clone(), path, lookup, upgrade)
                            .await;
                        return res;
                    }
                }
            }
        }
    }
}
