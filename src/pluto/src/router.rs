use std::{collections::HashMap, future::Future, pin::Pin};

use dyn_clone::{clone_trait_object, DynClone};
use matchit::{Match, Router as MatchRouter};

use crate::{
    http::{HttpRequest, HttpResponse},
    method::Method,
};

#[derive(Clone)]
pub(crate) struct HandlerContainer {
    pub(crate) upgrade: bool,
    pub(crate) handler: Box<dyn Handler>,
}

#[derive(Clone)]
pub struct Router {
    prefix: String,
    trees: HashMap<Method, MatchRouter<HandlerContainer>>,
    pub(crate) handle_options: bool,
    pub(crate) global_options: Option<HandlerContainer>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            prefix: String::from(""),
            trees: HashMap::new(),
            handle_options: true,
            global_options: None,
        }
    }

    pub fn set_global_prefix(&mut self, p: String) -> &mut Self {
        self.prefix = p;
        self
    }

    fn handle(
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

    pub fn get(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::GET, handler)
    }
    pub fn head(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::HEAD, handler)
    }
    pub fn options(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::OPTIONS, handler)
    }
    pub fn post(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::POST, handler)
    }
    pub fn put(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::PUT, handler)
    }
    pub fn patch(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::PATCH, handler)
    }
    pub fn delete(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static,
    ) -> &mut Self {
        self.handle(path, upgrade, Method::DELETE, handler)
    }

    pub fn handle_options(mut self) -> Self {
        self.handle_options = true;
        self
    }

    pub fn global_options(mut self, upgrade: bool, handler: impl Handler + 'static) -> Self {
        self.global_options = Some(HandlerContainer {
            handler: Box::new(handler),
            upgrade: upgrade,
        });
        self
    }

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
    fn handle(
        &self,
        req: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpResponse>> + Send + Sync>> {
        Box::pin(self(req))
    }
}
