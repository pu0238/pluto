use std::{collections::HashMap, future::Future, pin::Pin};

use dyn_clone::{clone_trait_object, DynClone};
use matchit::Match;

use crate::{method::Method, http::{HttpRequest, HttpResponse}};


#[derive(Clone)]
pub struct Router {
    prefix: String,
    trees: HashMap<Method, matchit::Router<HandlerContainer>>,
}

#[derive(Clone)]
pub struct HandlerContainer {
    pub upgrade: bool,
    pub handler: Box<dyn Handler>,
}

impl Router{
    pub fn new() -> Self {
        Self {
            prefix: String::from(""),
            trees: HashMap::new()
        }
    }

    pub fn set_global_prefix(&mut self, p: String) -> &mut Self{
        self.prefix = p;
        self
    }

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

        match self.trees
            .entry(method)
            .or_default()
            .insert(
                global_path,
                HandlerContainer {
                    handler: Box::new(handler),
                    upgrade: upgrade
                }
            ) {
                Err(err) => panic!("\nERROR: {}\n", err),
                Ok(_) => {}
            }
        self
    }

    pub fn lookup<'a>(
        &'a self,
        method: Method,
        path: &'a str,
    ) -> Result<Match<&HandlerContainer>, String> {
        if let Some(tree_at_path) = self.trees.get(&method) {
            if let Ok(match_result) = tree_at_path.at(path) {
                return Ok(match_result)
            }
        }

        if path == "" {
            return Err(
                format!("Cannot {} {}", method, "/")
            );
        }
        return Err(
            format!("Cannot {} {}", method, path)
        );
    }

    pub fn get(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static
    ) -> &mut Self{
        self.handle(path, upgrade, Method::GET, handler)
    }
    pub fn head(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static
    ) -> &mut Self {
        self.handle(path, upgrade, Method::HEAD, handler)
    }
    pub fn options(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static
    ) -> &mut Self {
        self.handle(path, upgrade, Method::OPTIONS, handler)
    }
    pub fn post(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static
    ) -> &mut Self {
        self.handle(path, upgrade, Method::POST, handler)
    }
    pub fn put(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static
    ) -> &mut Self {
        self.handle(path, upgrade, Method::PUT, handler)
    }
    pub fn patch(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static
    ) -> &mut Self{
        self.handle(path, upgrade, Method::PATCH, handler)
    }
    pub fn delete(
        &mut self,
        path: &str,
        upgrade: bool,
        handler: impl Handler + 'static
    ) -> &mut Self {
        self.handle(path, upgrade, Method::DELETE, handler)
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