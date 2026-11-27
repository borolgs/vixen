//! Request and response headers supported by htmx 4 core.
//!
//! Constants are also available directly under [`crate::hx`]. Extension-only
//! headers and headers removed in htmx 4 aren't included.

use axum::http::HeaderName;

/// `true` for requests made by htmx.
pub const HX_REQUEST: HeaderName = HeaderName::from_static("hx-request");
/// `full` when targeting `body` or using `hx-select`; `partial` otherwise.
pub const HX_REQUEST_TYPE: HeaderName = HeaderName::from_static("hx-request-type");
/// The browser's current URL when the request starts.
pub const HX_CURRENT_URL: HeaderName = HeaderName::from_static("hx-current-url");
/// The source element as `tag#id` or `tag`, with a percent-encoded id.
pub const HX_SOURCE: HeaderName = HeaderName::from_static("hx-source");
/// The target element as `tag#id` or `tag`, with a percent-encoded id.
pub const HX_TARGET: HeaderName = HeaderName::from_static("hx-target");
/// `true` for a boosted navigation request.
pub const HX_BOOSTED: HeaderName = HeaderName::from_static("hx-boosted");
/// `true` for a history restoration request.
pub const HX_HISTORY_RESTORE_REQUEST: HeaderName =
    HeaderName::from_static("hx-history-restore-request");

/// Dispatches client-side events after the swap completes.
pub const HX_TRIGGER: HeaderName = HeaderName::from_static("hx-trigger");
/// Navigates through htmx without a full page reload.
pub const HX_LOCATION: HeaderName = HeaderName::from_static("hx-location");
/// Navigates by setting `location.href`.
pub const HX_REDIRECT: HeaderName = HeaderName::from_static("hx-redirect");
/// Reloads the page when set to `true`.
pub const HX_REFRESH: HeaderName = HeaderName::from_static("hx-refresh");
/// Overrides the swap target.
pub const HX_RETARGET: HeaderName = HeaderName::from_static("hx-retarget");
/// Overrides the swap style.
pub const HX_RESWAP: HeaderName = HeaderName::from_static("hx-reswap");
/// Overrides the response content selection.
pub const HX_RESELECT: HeaderName = HeaderName::from_static("hx-reselect");
/// Replaces the current browser history URL, or disables replacement with `false`.
pub const HX_REPLACE_URL: HeaderName = HeaderName::from_static("hx-replace-url");
/// Pushes a browser history URL, or disables pushing with `false`.
pub const HX_PUSH_URL: HeaderName = HeaderName::from_static("hx-push-url");
