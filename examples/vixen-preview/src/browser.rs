use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use js_sys::{Array, Uint8Array};
use tower::ServiceExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Response, ResponseInit};

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

#[wasm_bindgen(start)]
pub fn start() {
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(tracing_web::MakeWebConsoleWriter::new())
        .init();
}

/// Dispatch a page-local request without a socket or a server runtime.
#[wasm_bindgen]
pub async fn handle_request(path: String, request: web_sys::Request) -> Result<Response, JsValue> {
    let is_head = request.method() == "HEAD";
    let mut builder = Request::builder()
        .method(request.method().as_str())
        .uri(path);

    // request headers
    let entries = js_sys::try_iter(&request.headers())?
        .ok_or_else(|| JsError::new("Request headers aren't iterable"))?;
    for entry in entries {
        let pair = Array::from(&entry?);
        let name = pair
            .get(0)
            .as_string()
            .ok_or_else(|| JsError::new("Invalid request header name"))?;
        let value = pair
            .get(1)
            .as_string()
            .ok_or_else(|| JsError::new("Invalid request header value"))?;
        builder = builder.header(name, value);
    }

    let bytes = JsFuture::from(request.array_buffer()?).await?;
    let request = builder
        .body(Body::from(Uint8Array::new(&bytes).to_vec()))
        .map_err(|error| JsError::new(&error.to_string()))?;

    let response = match super::router().oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    };
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, MAX_RESPONSE_BYTES)
        .await
        .map_err(|error| JsError::new(&error.to_string()))?;

    // response headers
    let headers = Headers::new()?;
    for (name, value) in &parts.headers {
        headers.append(
            name.as_str(),
            value
                .to_str()
                .map_err(|error| JsError::new(&error.to_string()))?,
        )?;
    }

    let init = ResponseInit::new();
    init.set_status(parts.status.as_u16());
    init.set_status_text(parts.status.canonical_reason().unwrap_or(""));
    init.set_headers(&headers);

    // Fetch forbids bodies on these statuses, even an empty byte array.
    let body = if is_head || matches!(parts.status.as_u16(), 204 | 205 | 304) {
        None
    } else {
        Some(Uint8Array::from(bytes.as_ref()))
    };
    Response::new_with_opt_js_u8_array_and_init(body.as_ref(), &init)
}
