/// A helper macro for quickly rendering a view inside a route call.
///
/// # Example
///
///
#[macro_export]
macro_rules! render_view {
    (
        $view:path
        $(, $arg:expr)*
    ) => {
        let mut headers = HashMap::from([
            ("Content-Type".to_string(), "text/html".to_string()),
        ]);
        let mut buffer: Vec<u8> = Vec::new();
        $view(&mut buffer$(, $arg)*).unwrap();
        return Ok(ic_pluto::http::HttpResponse {
            status_code: 200,
            headers,
            body: ic_pluto::http::HttpBody::String(String::from_utf8(buffer).unwrap()),
        })
    };
}
