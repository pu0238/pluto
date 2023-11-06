use crate::{all_or_some::AllOrSome, http::HttpResponse, method::Method};
use std::ops::Deref;

#[derive(Eq, PartialEq, Debug)]
pub struct Cors {
    allow_origin: Option<AllOrSome<String>>,
    allow_methods: Vec<Method>,
    allow_headers: Vec<String>,
    allow_credentials: bool,
    expose_headers: Vec<String>,
    max_age: Option<usize>,
    vary_origin: bool,
}

impl Cors {
    /// Create an empty `Cors`
    pub fn new() -> Self {
        Self {
            allow_origin: None,
            allow_headers: vec![],
            allow_methods: vec![],
            allow_credentials: false,
            expose_headers: vec![],
            max_age: None,
            vary_origin: false,
        }
    }

    /// Consumes the `Response` and return an altered response with origin and `vary_origin` set
    pub fn allow_origin(mut self, origin: &str) -> Self {
        self.allow_origin = Some(AllOrSome::Some(origin.to_string()));
        self
    }

    /// Consumes the `Response` and return an altered response with origin set to "*"
    pub fn any(mut self) -> Self {
        self.allow_origin = Some(AllOrSome::All);
        self
    }

    /// Consumes the Response and set credentials
    pub fn credentials(mut self, value: bool) -> Self {
        self.allow_credentials = value;
        self
    }

    /// Consumes the CORS, set expose_headers to
    /// passed headers and returns changed CORS
    pub fn exposed_headers(mut self, headers: Vec<&str>) -> Self {
        self.expose_headers = headers.iter().map(|s| (*s).to_string().into()).collect();
        self
    }

    /// Consumes the CORS, set allow_headers to
    /// passed headers and returns changed CORS
    pub fn allow_headers(mut self, headers: Vec<&str>) -> Self {
        self.allow_headers = headers.iter().map(|s| (*s).to_string().into()).collect();
        self
    }

    /// Consumes the CORS, set max_age to
    /// passed value and returns changed CORS
    pub fn max_age(mut self, value: Option<usize>) -> Self {
        self.max_age = value;
        self
    }

    /// Consumes the CORS, set allow_methods to
    /// passed methods and returns changed CORS
    pub fn allow_methods(mut self, methods: Vec<Method>) -> Self {
        self.allow_methods = methods.clone();
        self
    }

    /// Merge CORS headers with an existing `rocket::Response`.
    ///
    /// This will overwrite any existing CORS headers
    pub fn merge(&self, response: &mut HttpResponse) {
        let origin = match self.allow_origin {
            None => {
                // This is not a CORS response
                return;
            }
            Some(ref origin) => origin,
        };

        let origin = match *origin {
            AllOrSome::All => "*".to_string(),
            AllOrSome::Some(ref origin) => origin.to_string(),
        };

        response.add_raw_header("Access-Control-Allow-Origin", origin);

        if self.allow_credentials {
            response.add_raw_header("Access-Control-Allow-Credentials", "true".to_string());
        }

        if !self.expose_headers.is_empty() {
            let headers: Vec<String> = self
                .expose_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            response.add_raw_header("Access-Control-Expose-Headers", headers);
        }

        if !self.allow_headers.is_empty() {
            let headers: Vec<String> = self
                .allow_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            response.add_raw_header("Access-Control-Allow-Headers", headers);
        }

        if !self.allow_methods.is_empty() {
            let methods: Vec<_> = self.allow_methods.iter().map(|m| m.as_str()).collect();
            let methods = methods.join(", ");

            response.add_raw_header("Access-Control-Allow-Methods", methods);
        }

        if self.max_age.is_some() {
            let max_age = self.max_age.unwrap();
            response.add_raw_header("Access-Control-Max-Age", max_age.to_string());
        }

        if self.vary_origin {
            response.add_raw_header("Vary", "Origin".to_string());
        }
    }
}
