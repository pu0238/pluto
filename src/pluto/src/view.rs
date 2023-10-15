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
        let mut headers = HashMap::new();
        headers.insert(String::from("Content-Type"), String::from("text/html"));
        let mut buffer: Vec<u8> = vec![];
        $view(&mut buffer$(, $arg)*).unwrap();
        return Ok(pluto::http::HttpResponse {
            status_code: 200,
            headers,
            body: pluto::http::HttpBody::String(String::from_utf8(buffer).unwrap()),
        })
    };
}
