//! The htmx 4 types used by vixen, plus all core header constants.

use std::convert::Infallible;

use axum::{
    extract::FromRequestParts,
    http::{HeaderValue, StatusCode, Uri, header::InvalidHeaderValue, request::Parts},
    response::{IntoResponseParts, ResponseParts},
};
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

pub mod headers;
pub use headers::*;

/// A built-in `hx-swap` or `HX-Reswap` style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapOption {
    /// Replace the target's contents with HTML.
    InnerHtml,
    /// Replace the target element with HTML.
    OuterHtml,
    /// Insert before the target element.
    BeforeBegin,
    /// Insert before the target's first child.
    AfterBegin,
    /// Insert after the target's last child.
    BeforeEnd,
    /// Insert after the target element.
    AfterEnd,
    /// Delete the target element.
    Delete,
    /// Skip the main swap; targeted partials still run.
    None,
    /// Replace the target's text without parsing HTML.
    TextContent,
    /// Morph the target's contents while preserving element state.
    InnerMorph,
    /// Morph the target element while preserving element state.
    OuterMorph,
    /// Replace the target while preserving unchanged element state.
    OuterSync,
}

impl SwapOption {
    /// The value used in HTML attributes and HTTP headers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InnerHtml => "innerHTML",
            Self::OuterHtml => "outerHTML",
            Self::BeforeBegin => "beforebegin",
            Self::AfterBegin => "afterbegin",
            Self::BeforeEnd => "beforeend",
            Self::AfterEnd => "afterend",
            Self::Delete => "delete",
            Self::None => "none",
            Self::TextContent => "textContent",
            Self::InnerMorph => "innerMorph",
            Self::OuterMorph => "outerMorph",
            Self::OuterSync => "outerSync",
        }
    }
}

impl Serialize for SwapOption {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl From<SwapOption> for HeaderValue {
    fn from(value: SwapOption) -> Self {
        HeaderValue::from_static(value.as_str())
    }
}

/// A client-side event for [`crate::partial!`] or [`crate::HxPartial::event`].
///
/// htmx 4 dispatches `HX-Trigger` events after the swap. Payloads are passed
/// through unchanged; events without data use JSON `null`. This requires
/// htmx 4.0.1 or later, which fixes dispatching events with null detail.
#[derive(Debug, Clone, Serialize)]
pub struct HxEvent {
    /// The client-side event name.
    pub name: String,
    /// Event detail. htmx exposes scalar payloads under `event.detail.value`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl HxEvent {
    /// Creates an event without data.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: None,
        }
    }

    /// Serializes data into an event payload.
    pub fn new_with_data(
        name: impl Into<String>,
        data: impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            name: name.into(),
            data: Some(serde_json::to_value(data)?),
        })
    }
}

impl<T: Into<String>> From<T> for HxEvent {
    fn from(name: T) -> Self {
        Self::new(name)
    }
}

/// The browser URL from `HX-Current-URL`.
///
/// Missing or malformed headers yield `None`. The URL is client-provided;
/// don't use it as proof of origin or authorization.
#[derive(Debug, Clone)]
pub struct HxCurrentUrl(pub Option<Uri>);

impl<S: Send + Sync> FromRequestParts<S> for HxCurrentUrl {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let url = parts
            .headers
            .get(HX_CURRENT_URL)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());
        Ok(Self(url))
    }
}

/// Replaces the browser URL without adding a history entry.
///
/// Use `"false"` to suppress replacement. Invalid HTTP header values return
/// a `500 Internal Server Error` response rather than panicking.
#[derive(Debug, Clone)]
pub struct HxReplaceUrl(pub String);

impl IntoResponseParts for HxReplaceUrl {
    type Error = (StatusCode, String);

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let value = HeaderValue::from_maybe_shared(self.0)
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
        res.headers_mut().insert(HX_REPLACE_URL, value);
        Ok(res)
    }
}

pub(crate) fn trigger_header(
    events: Vec<HxEvent>,
) -> Result<Option<HeaderValue>, InvalidHeaderValue> {
    if events.is_empty() {
        return Ok(None);
    }
    // A JSON object also handles event names containing commas. The last
    // occurrence of a repeated name wins, as it did in axum-htmx.
    let events: Map<String, Value> = events
        .into_iter()
        .map(|event| (event.name, event.data.unwrap_or(Value::Null)))
        .collect();
    let json = Value::Object(events).to_string();
    // HTTP field values aren't UTF-8. Escape non-ASCII JSON characters so
    // browsers receive the original text, including surrogate pairs.
    let mut ascii = String::with_capacity(json.len());
    for ch in json.chars() {
        if ch.is_ascii() {
            ascii.push(ch);
        } else {
            for unit in ch.encode_utf16(&mut [0; 2]) {
                ascii.push_str(&format!("\\u{unit:04x}"));
            }
        }
    }
    HeaderValue::from_maybe_shared(ascii).map(Some)
}

#[cfg(test)]
mod tests;
