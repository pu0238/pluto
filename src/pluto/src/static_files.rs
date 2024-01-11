/// The main way to load the static files as ready-to-use routes in the application.
///
/// This ensures every file is accessible through HTTP GET requests by adding all of them to the router automatically.
///
/// # Example
///
/// The best way to use this macro is to include it in the bootstraping step for the router:
///
///
/// ```rust
/// #[post_upgrade]
/// fn post_upgrade() {
///     ROUTER.with(|r| {
///         let mut instance = controller::setup();
///         pluto::use_static_files!(instance);
///         *r.borrow_mut() = instance;
///     })
/// }
/// ````
#[macro_export]
macro_rules! use_static_files {
    (
        $router:path
    ) => {
        for file in crate::compiled::templates::statics::STATICS.iter() {
            $router.get(&format!("/static/{}", file.name), false, |_req| async {
                let bytes = file.content.clone();
                let mut headers: HashMap<String, String> = HashMap::new();
                headers.insert("Content-Type".to_string(), file.mime.to_string());
                return Ok(HttpResponse {
                    status_code: 200,
                    headers,
                    body: if file.mime.type_() == "text" || file.mime.subtype() == "json" {
                        pluto::http::HttpBody::String(String::from_utf8(bytes.to_vec()).unwrap())
                    } else {
                        bytes.to_owned().into()
                    },
                });
            });
        }
    };
}
