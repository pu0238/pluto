use std::{collections::HashMap, future::Future, pin::Pin};

use dyn_clone::{clone_trait_object, DynClone};
use matchit::{Match, Router as MatchRouter};

use crate::{
    http::{HttpRequest, HttpResponse},
    method::Method,
};

/// A container for a handler and a flag indicating whether the handler supports HTTP upgrades.
#[derive(Clone)]
pub(crate) struct HandlerContainer {
    pub(crate) upgrade: bool,
    pub(crate) handler: Box<dyn Handler>,
}

/// A router for HTTP requests.
/// The router is used to register handlers for different HTTP methods and paths.
#[derive(Clone)]
pub struct Router {
    prefix: String,
    trees: HashMap<Method, MatchRouter<HandlerContainer>>,
    pub(crate) handle_options: bool,
    pub(crate) global_options: Option<HandlerContainer>,
}

impl Router {
    /// Create a new router.
    /// The router is used to register handlers for different HTTP methods and paths.
    /// The router can be used as a handler for a server.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    ///
    /// let mut router = Router::new();
    /// ```
    pub fn new() -> Self {
        Self {
            prefix: String::from(""),
            trees: HashMap::new(),
            handle_options: true,
            global_options: None,
        }
    }

    /// Set a prefix for all paths registered on the router.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    ///
    /// let mut router = Router::new();
    /// router.set_global_prefix("/api".to_string());
    /// ```
    pub fn set_global_prefix(&mut self, p: String) -> &mut Self {
        self.prefix = p;
        self
    }

    /// Register a handler for a path and method.
    /// The handler is called for requests with a matching path and method.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use pluto::method::Method;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.handle("/hello", false, Method::GET, |req: HttpRequest| async move {
    ///     Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from GET",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn handle(
        &mut self,
        path: &str,
        upgrade: bool,
        method: Method,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        if !path.starts_with('/') {
            panic!("expect path beginning with '/', found: '{}'", path);
        }
        let mut global_path = self.prefix.to_owned() + path;
        if global_path.ends_with("/") {
            global_path.pop();
        }

        match self.trees.entry(method).or_default().insert(
            global_path,
            HandlerContainer {
                handler: Box::new(handler),
                upgrade: upgrade,
            },
        ) {
            Err(err) => panic!("\nERROR: {}\n", err),
            Ok(_) => {}
        }
        self
    }

    /// Lookup a handler for a path and method.
    /// The handler is called for requests with a matching path and method.
    pub(crate) fn lookup<'a>(
        &'a self,
        method: Method,
        path: &'a str,
    ) -> Result<Match<&HandlerContainer>, String> {
        if let Some(tree_at_path) = self.trees.get(&method) {
            if let Ok(match_result) = tree_at_path.at(path) {
                return Ok(match_result);
            }
        }

        if path == "" {
            return Err(format!("Cannot {} {}", method, "/"));
        }
        return Err(format!("Cannot {} {}", method, path));
    }

    /// Register a handler for GET requests at a path.
    /// The handler is called for requests with the GET method and a matching path.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use pluto::method::Method;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.get("/hello", false, |req: HttpRequest| async move {
    ///     Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from GET",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn get(&mut self, path: &str, upgrade: bool, handler: impl Handler + 'static) -> &mut Self {
        self.handle(path, upgrade, Method::GET, handler)
    }

    /// Register a handler for HEAD requests at a path.
    /// The handler is called for requests with the HEAD method and a matching path.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use pluto::method::Method;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.head("/hello", false, |req: HttpRequest| async move {
    ///     Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from HEAD",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn head(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::HEAD, handler)
    }

    /// Register a handler for OPTIONS requests at a path.
    /// The handler is called for requests with the OPTIONS method and a matching path.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use pluto::method::Method;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.options("/hello", false, |req: HttpRequest| async move {
    ///     Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from OPTIONS",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn options(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::OPTIONS, handler)
    }

    /// Register a handler for POST requests at a path.
    /// The handler is called for requests with the POST method and a matching path.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use pluto::method::Method;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.post("/hello", false, |req: HttpRequest| async move {
    ///     Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from POST",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn post(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::POST, handler)
    }

    /// Register a handler for PUT requests at a path.
    /// The handler is called for requests with the PUT method and a matching path.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use pluto::method::Method;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.put("/hello", false, |req: HttpRequest| async move {
    ///     Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from PUT",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn put(&mut self, path: &str, upgrade: bool, handler: impl Handler + 'static) -> &mut Self {
        self.handle(path, upgrade, Method::PUT, handler)
    }

    /// Register a handler for PATCH requests at a path.
    /// The handler is called for requests with the PATCH method and a matching path.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use pluto::method::Method;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.patch("/hello", false, |req: HttpRequest| async move {
    ///     Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from PATCH",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn patch(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::PATCH, handler)
    }

    /// Register a handler for DELETE requests at a path.
    /// The handler is called for requests with the DELETE method and a matching path.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use pluto::method::Method;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.delete("/hello", false, |req: HttpRequest| async move {
    ///     Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from DELETE",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn delete(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::DELETE, handler)
    }

    /// Allow the router to handle OPTIONS requests.
    /// If enabled, the router will automatically respond to OPTIONS requests with the allowed methods for a path.
    /// If disabled, the router will respond to OPTIONS requests with a 404.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    ///
    /// let mut router = Router::new();
    /// router.handle_options(true);
    /// ```
    pub fn handle_options(&mut self, handle: bool) {
        self.handle_options = handle;
    }

    /// Register a default handler for not registered requests.
    /// The handler is called for requests when router can't matching path or method to any handler.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.global_options(false, |req: HttpRequest| async move {
    ///    Ok(HttpResponse {
    ///         status_code: 404,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 404,
    ///             "message": "Not Found",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// ```
    pub fn global_options(mut self, upgrade: bool, handler: impl Handler + 'static) -> Self {
        self.global_options = Some(HandlerContainer {
            handler: Box::new(handler),
            upgrade: upgrade,
        });
        self
    }

    /// Get the allowed methods for a path.
    /// # Examples
    ///
    /// ``` rust
    /// use pluto::router::Router;
    /// use pluto::http::{HttpRequest, HttpResponse};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut router = Router::new();
    /// router.get("/hello", false, |req: HttpRequest| async move {
    ///    Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from GET",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// router.post("/hello", false, |req: HttpRequest| async move {
    ///   Ok(HttpResponse {
    ///         status_code: 200,
    ///         headers: HashMap::new(),
    ///         body: json!({
    ///             "statusCode": 200,
    ///             "message": "Hello World from POST",
    ///         })
    ///         .into(),
    ///     })
    /// });
    /// let mut allowed = router.allowed("/hello");
    /// allowed.sort();
    /// assert_eq!(allowed, vec!["GET", "OPTIONS", "POST"]);
    pub fn allowed(&self, path: &str) -> Vec<&str> {
        let mut allowed = match path {
            "*" => {
                let mut allowed = Vec::with_capacity(self.trees.len());
                for method in self
                    .trees
                    .keys()
                    .filter(|&method| method != Method::OPTIONS)
                {
                    allowed.push(method.as_ref());
                }
                allowed
            }
            _ => self
                .trees
                .keys()
                .filter(|&method| method != Method::OPTIONS)
                .filter(|&method| {
                    self.trees
                        .get(method)
                        .map(|node| node.at(path).is_ok())
                        .unwrap_or(false)
                })
                .map(AsRef::as_ref)
                .collect::<Vec<_>>(),
        };

        if !allowed.is_empty() {
            allowed.push(Method::OPTIONS.as_ref())
        }

        allowed
    }
}

clone_trait_object!(Handler);
pub trait Handler: Send + Sync + DynClone {
    /// Handle a request.
    /// The handler is called for requests with a matching path and method.
    fn handle(
        &self,
        req: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpResponse>> + Send + Sync>>;
}

impl<F, R> Handler for F
where
    F: Fn(HttpRequest) -> R + Send + Sync + DynClone,
    R: Future<Output = Result<HttpResponse, HttpResponse>> + Send + Sync + 'static,
{
    /// Handle a request.
    /// The handler is called for requests with a matching path and method.
    fn handle(
        &self,
        req: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpResponse>> + Send + Sync>> {
        Box::pin(self(req))
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;
    use crate::http::{HttpRequest, HttpResponse};
    use crate::method::Method;

    #[test]
    fn test_router() {
        let mut router = Router::new();
        router.get("/hello", false, |_req: HttpRequest| async move {
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
        router.post("/hello", false, |_req: HttpRequest| async move {
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
        router.put("/hello", false, |_req: HttpRequest| async move {
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
        router.patch("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "statusCode": 200,
                    "message": "Hello World from PATCH",
                })
                .into(),
            })
        });
        router.delete("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "statusCode": 200,
                    "message": "Hello World from DELETE",
                })
                .into(),
            })
        });
        router.head("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "statusCode": 200,
                    "message": "Hello World from HEAD",
                })
                .into(),
            })
        });
        router.options("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "statusCode": 200,
                    "message": "Hello World from OPTIONS",
                })
                .into(),
            })
        });

        let mut allowed = router.allowed("/hello");
        allowed.sort();
        assert_eq!(
            allowed,
            vec![
                Method::DELETE.as_ref(),
                Method::GET.as_ref(),
                Method::HEAD.as_ref(),
                Method::OPTIONS.as_ref(),
                Method::PATCH.as_ref(),
                Method::POST.as_ref(),
                Method::PUT.as_ref(),
            ]
        );
    }

    #[test]
    fn test_router_prefix() {
        let mut router = Router::new();
        router.set_global_prefix("/api".to_string());
        router.get("/hello", false, |_req: HttpRequest| async move {
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
        router.post("/hello", false, |_req: HttpRequest| async move {
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
        router.put("/hello", false, |_req: HttpRequest| async move {
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
        router.patch("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "statusCode": 200,
                    "message": "Hello World from PATCH",
                })
                .into(),
            })
        });
        router.delete("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "statusCode": 200,
                    "message": "Hello World from DELETE",
                })
                .into(),
            })
        });
        router.head("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "statusCode": 200,
                    "message": "Hello World from HEAD",
                })
                .into(),
            })
        });
        router.options("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "statusCode": 200,
                    "message": "Hello World from OPTIONS",
                })
                .into(),
            })
        });

        let mut allowed = router.allowed("/api/hello");
        allowed.sort();
        assert_eq!(
            allowed,
            vec![
                Method::DELETE.as_ref(),
                Method::GET.as_ref(),
                Method::HEAD.as_ref(),
                Method::OPTIONS.as_ref(),
                Method::PATCH.as_ref(),
                Method::POST.as_ref(),
                Method::PUT.as_ref(),
            ]
        );
    }

    #[tokio::test]
    async fn test_lookup_works() {
        let mut router = Router::new();
        let response = HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "message": "Hello World from GET",
            })
            .into(),
        };
        router.get("/hello", false, |_req: HttpRequest| async move {
            Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "message": "Hello World from GET",
                })
                .into(),
            })
        });

        let allowed = router.lookup(Method::GET, "/hello").unwrap();
        let result = allowed
            .value
            .handler
            .handle(
                crate::http::RawHttpRequest {
                    method: "GET".to_string(),
                    url: "http:://localhost:8080/hello".to_string(),
                    headers: Vec::new(),
                    body: Vec::new(),
                }
                .into(),
            )
            .await
            .unwrap();

        assert_eq!(
            result,
            HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: json!({
                    "message": "Hello World from GET",
                })
                .into(),
            }
        );
    }
}
