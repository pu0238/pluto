use crate::{
    cors::Cors,
    method::Method,
    router::{HandlerContainer, Router},
};
use candid::{CandidType, Deserialize};
use matchit::{Match, Params as MatchitParams};
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, str::FromStr};

/// HeaderField is the type of the header of the request.
#[derive(CandidType, Deserialize, Clone)]
pub struct HeaderField(String, String);

/// RawHttpRequest is the request type that is sent by the client.
/// It is a raw version of HttpRequest. It is compatible with the Candid type.
/// It is used in the 'http_request' and 'http_request_update' function of the canister and it is provided by the IC.
/// It is converted to HttpRequest before it is used in the handler.
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
            params: HashMap::new(),
            path: String::new(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
/// HttpRequest is the request type that is available in handler.
/// It is a more user-friendly version of RawHttpRequest
/// It is used in handler to allow user to process the request.
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
    pub path: String,
}

impl HttpRequest {
    pub fn body_into_struct<T: for<'a> Deserialize<'a>>(&self) -> Result<T, HttpResponse> {
        serde_json::from_slice(&self.body).map_err(|msg| HttpResponse {
            status_code: 400,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 400,
                "message": msg.to_string(),
            })
            .into(),
        })
    }

    pub fn params_into_struct<T: for<'a> Deserialize<'a>>(&self) -> Result<T, HttpResponse> {
        let json = serde_json::json!(&self.params);
        serde_json::from_value(json).map_err(|msg| HttpResponse {
            status_code: 400,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 400,
                "message": msg.to_string(),
            })
            .into(),
        })
    }
}

/// RawHttpResponse is the response type that is sent back to the client.
/// It is a raw version of HttpResponse. It is compatible with the Candid type.
#[derive(CandidType, Deserialize)]
pub struct RawHttpResponse {
    pub(crate) status_code: u16,
    pub(crate) headers: HashMap<String, String>,
    #[serde(with = "serde_bytes")]
    pub(crate) body: Vec<u8>,
    pub(crate) upgrade: Option<bool>,
}

impl RawHttpResponse {
    /// Set the upgrade flag of the response.
    fn set_upgrade(&mut self, upgrade: bool) {
        self.upgrade = Some(upgrade);
    }

    /// Enrich the header of the response depending on the content the body.
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum HttpBody {
    Value(Value),
    String(String),
    Raw(Vec<u8>),
}

impl From<HttpBody> for Vec<u8> {
    fn from(b: HttpBody) -> Self {
        return match b {
            HttpBody::Value(json) => json.to_string().into_bytes().into(),
            HttpBody::String(string) => string.into_bytes().into(),
            HttpBody::Raw(vec) => vec,
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

impl From<Vec<u8>> for HttpBody {
    fn from(value: Vec<u8>) -> Self {
        Self::Raw(value)
    }
}

/// HttpResponse is the response type that is available in handler.
/// It is a more user-friendly version of RawHttpResponse
/// After the handler is executed, it is converted to RawHttpResponse.
#[derive(Debug, PartialEq, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: HttpBody,
}

impl HttpResponse {
    /// Add a header to the response.
    /// If the header already exists, it will be overwritten.
    pub fn add_raw_header(&mut self, key: &str, value: String) {
        self.headers.insert(key.to_string(), value);
    }

    /// Remove a header from the response.
    /// If the header does not exist, nothing will happen.
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

/// This macro is used to create a new instance of HttpServe with given router.
/// It is used in the 'http_request' and 'http_request_update' function of the canister.
/// This macro handles routing from not upgradable request to upgradable request.
///
/// # Example
///
/// ```rust
/// use ic_cdk::{query, update};
///
/// use pluto::router::Router;
/// use pluto::http_serve_router;
/// use pluto::http::{RawHttpRequest, RawHttpResponse};
/// use pluto::http::HttpServe;
///
/// #[query]
/// async fn http_request(req: RawHttpRequest) -> RawHttpResponse {
///     let router = setup_router();
///     http_serve_router!(router).serve(req).await
/// }
///
/// #[update]
/// async fn http_request_update(req: RawHttpRequest) -> RawHttpResponse {
///     let router = setup_router();
///     http_serve_router!(router).serve(req).await
/// }
///
/// fn setup_router() -> Router {
///    Router::new()
/// }
/// ```
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

/// This macro is used to create a new instance of HttpServe.
/// It is used in the 'http_request' and 'http_request_update' function of the canister.
/// This macro handles routing from not upgradable request to upgradable request.
///
/// # Example
///
/// ```rust
/// use ic_cdk::{query, update};
///
/// use pluto::router::Router;
/// use pluto::http_serve;
/// use pluto::http::{RawHttpRequest, RawHttpResponse};
/// use pluto::http::HttpServe;
///
/// #[query]
/// async fn http_request(req: RawHttpRequest) -> RawHttpResponse {
///     bootstrap(http_serve!(), req).await
/// }
///
/// #[update]
/// async fn http_request_update(req: RawHttpRequest) -> RawHttpResponse {
///     bootstrap(http_serve!(), req).await
/// }
///
/// async fn bootstrap(mut app: HttpServe, req: RawHttpRequest) -> RawHttpResponse {
///     let router = Router::new();
///     app.set_router(router);
///     app.serve(req).await
/// }
/// ```
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

/// HttpServe is the main struct of the Pluto library.
/// It is used to create a new instance of HttpServe.
/// It is used in the 'http_request' and 'http_request_update' function of the canister.
/// This struct handles routing from not upgradable request to upgradable request.
/// It also handles CORS.
pub struct HttpServe {
    router: Router,
    cors_policy: Option<Cors>,
    is_query: bool,
}

impl HttpServe {
    /// Create a new instance of HttpServe depending on the function name.
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

    /// Create a new instance of HttpServe with given router.
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

    /// Set the router of the HttpServe.
    pub fn set_router(&mut self, r: Router) {
        self.router = r;
    }

    /// Add a handler to the router.
    /// The handler will be executed if the request do matches any method and path.
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

    /// Predefined server error response.
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

    /// Predefined not found error response.
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

    fn params_to_string(params: MatchitParams) -> HashMap<String, String> {
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
        let mut req: HttpRequest = req.into();
        req.path = String::from(path);
        req.params = Self::params_to_string(lookup.params);
        let handle_res = lookup.value.handler.handle(req).await;
        let mut res = Self::unwrap_response(handle_res);
        self.use_res_plugins(&mut res);
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

    fn use_res_plugins(self, res: &mut HttpResponse) {
        self.add_cors_to_res(res);
    }

    fn add_cors_to_res(self, res: &mut HttpResponse) {
        if let Some(cors) = self.cors_policy {
            cors.merge(res)
        }
    }

    /// Set the CORS policy of the HttpServe.
    /// ```rust
    /// use ic_cdk::{query, update};
    ///
    /// use pluto::router::Router;
    /// use pluto::http_serve;
    /// use pluto::http::{RawHttpRequest, RawHttpResponse};
    /// use pluto::http::HttpServe;
    /// use pluto::method::Method;
    /// use pluto::cors::Cors;
    ///
    /// #[query]
    /// async fn http_request(req: RawHttpRequest) -> RawHttpResponse {
    ///     bootstrap(http_serve!(), req).await
    /// }
    ///
    /// #[update]
    /// async fn http_request_update(req: RawHttpRequest) -> RawHttpResponse {
    ///     bootstrap(http_serve!(), req).await
    /// }
    ///
    /// async fn bootstrap(mut app: HttpServe, req: RawHttpRequest) -> RawHttpResponse {
    ///     let router = Router::new();
    ///     let cors = Cors::new()
    ///         .allow_origin("*")
    ///         .allow_methods(vec![Method::POST, Method::PUT])
    ///         .allow_headers(vec!["Content-Type", "Authorization"])
    ///         .max_age(Some(3600));
    ///
    ///     app.set_router(router);
    ///     app.use_cors(cors);
    ///     app.serve(req).await
    /// }
    /// ```
    pub fn use_cors(&mut self, cors_policy: Cors) {
        self.cors_policy = Some(cors_policy);
    }

    /// Serve the request.
    /// It will return a RawHttpResponse.
    /// It will return an internal server error if the request is not valid.
    /// It will return a not found error if the request does not match any method and path.
    /// ```rust
    /// use ic_cdk::{query, update};
    ///
    /// use pluto::router::Router;
    /// use pluto::http_serve;
    /// use pluto::http::{RawHttpRequest, RawHttpResponse};
    /// use pluto::http::HttpServe;
    ///
    /// #[query]
    /// async fn http_request(req: RawHttpRequest) -> RawHttpResponse {
    ///     bootstrap(http_serve!(), req).await
    /// }
    ///
    /// #[update]
    /// async fn http_request_update(req: RawHttpRequest) -> RawHttpResponse {
    ///     bootstrap(http_serve!(), req).await
    /// }
    ///
    /// async fn bootstrap(mut app: HttpServe, req: RawHttpRequest) -> RawHttpResponse {
    ///     let router = Router::new();
    ///     app.set_router(router);
    ///     app.serve(req).await
    /// }
    /// ```
    pub async fn serve(self, req: RawHttpRequest) -> RawHttpResponse {
        match Method::from_str(req.method.as_ref()) {
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
                                        self.use_res_plugins(&mut res);
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
