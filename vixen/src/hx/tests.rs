use axum::{
    body::Body,
    http::{Request, header::VARY},
    response::IntoResponse,
};
use maud::{Render, html};
use serde_json::json;

use super::*;
use crate::{HxAction, HxPartial};

#[test]
fn core_headers_match_the_htmx_4_protocol() {
    for (header, name) in [
        (HX_REQUEST, "hx-request"),
        (HX_REQUEST_TYPE, "hx-request-type"),
        (HX_CURRENT_URL, "hx-current-url"),
        (HX_SOURCE, "hx-source"),
        (HX_TARGET, "hx-target"),
        (HX_BOOSTED, "hx-boosted"),
        (HX_HISTORY_RESTORE_REQUEST, "hx-history-restore-request"),
        (HX_TRIGGER, "hx-trigger"),
        (HX_LOCATION, "hx-location"),
        (HX_REDIRECT, "hx-redirect"),
        (HX_REFRESH, "hx-refresh"),
        (HX_RETARGET, "hx-retarget"),
        (HX_RESWAP, "hx-reswap"),
        (HX_RESELECT, "hx-reselect"),
        (HX_REPLACE_URL, "hx-replace-url"),
        (HX_PUSH_URL, "hx-push-url"),
    ] {
        assert_eq!(header.as_str(), name);
    }
}

#[test]
fn swap_styles_match_in_headers_json_actions_and_partials() {
    for (swap, name) in [
        (SwapOption::InnerHtml, "innerHTML"),
        (SwapOption::OuterHtml, "outerHTML"),
        (SwapOption::BeforeBegin, "beforebegin"),
        (SwapOption::AfterBegin, "afterbegin"),
        (SwapOption::BeforeEnd, "beforeend"),
        (SwapOption::AfterEnd, "afterend"),
        (SwapOption::Delete, "delete"),
        (SwapOption::None, "none"),
        (SwapOption::TextContent, "textContent"),
        (SwapOption::InnerMorph, "innerMorph"),
        (SwapOption::OuterMorph, "outerMorph"),
        (SwapOption::OuterSync, "outerSync"),
    ] {
        assert_eq!(swap.as_str(), name);
        assert_eq!(HeaderValue::from(swap), name);
        assert_eq!(serde_json::to_value(swap).unwrap(), json!(name));
        let attr = format!(r#"hx-swap="{name}""#);
        assert!(HxAction::new("/save").swap(swap).render().0.contains(&attr));
        assert!(
            HxPartial::new()
                .target_swap("#result", swap, html! {})
                .render()
                .0
                .contains(&attr)
        );
    }
}

#[test]
fn event_payloads_are_preserved_including_null_and_scalars() {
    let events = vec![
        HxEvent::new("empty"),
        HxEvent::new_with_data("null", Value::Null).unwrap(),
        HxEvent::new_with_data("text", "saved").unwrap(),
        HxEvent::new_with_data("number", 42).unwrap(),
        HxEvent::new_with_data("bool", true).unwrap(),
        HxEvent::new_with_data("array", [1, 2]).unwrap(),
        HxEvent::new_with_data("targeted", json!({"target": "#dialog", "id": 7})).unwrap(),
    ];
    let header = trigger_header(events).unwrap().unwrap();
    let value: Value = serde_json::from_str(header.to_str().unwrap()).unwrap();
    assert_eq!(
        value,
        json!({
            "empty": null, "null": null, "text": "saved", "number": 42,
            "bool": true, "array": [1, 2], "targeted": {"target": "#dialog", "id": 7}
        })
    );
    assert!(trigger_header(vec![]).unwrap().is_none());
}

#[test]
fn event_names_and_unicode_payloads_are_json_escaped_for_http() {
    let name = "сохранено,\"\r\n🚢";
    let data = json!({"message": "готово\r\n🚢"});
    let header = trigger_header(vec![HxEvent::new_with_data(name, &data).unwrap()])
        .unwrap()
        .unwrap();
    let encoded = header.to_str().unwrap();
    assert!(encoded.is_ascii());
    assert!(!encoded.contains(['\r', '\n']));
    let decoded: Value = serde_json::from_str(encoded).unwrap();
    assert_eq!(decoded[name], data);
    assert_eq!(decoded.as_object().unwrap().len(), 1);
}

#[test]
fn the_last_event_with_a_given_name_wins() {
    let header = trigger_header(vec![
        HxEvent::new_with_data("saved", 1).unwrap(),
        HxEvent::new_with_data("saved", 2).unwrap(),
    ])
    .unwrap()
    .unwrap();
    assert_eq!(header, r#"{"saved":2}"#);
}

#[test]
fn event_payload_serialization_errors_are_returned() {
    struct Invalid;
    impl Serialize for Invalid {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("cannot serialize payload"))
        }
    }
    assert!(HxEvent::new_with_data("saved", Invalid).is_err());
}

#[tokio::test]
async fn current_url_preserves_optional_extractor_behavior() {
    for (header, expected) in [
        (None, None),
        (
            Some("https://example.com/todos?q=milk#list"),
            Some("q=milk"),
        ),
        (Some("/todos?q=tea"), Some("q=tea")),
        (Some("not a uri"), None),
    ] {
        let mut request = Request::builder();
        if let Some(value) = header {
            request = request.header(HX_CURRENT_URL, value);
        }
        let (mut parts, _) = request.body(Body::empty()).unwrap().into_parts();
        let HxCurrentUrl(uri) = HxCurrentUrl::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(uri.as_ref().and_then(Uri::query), expected);
    }
    let (mut parts, _) = Request::new(Body::empty()).into_parts();
    parts
        .headers
        .insert(HX_CURRENT_URL, HeaderValue::from_bytes(b"\xff").unwrap());
    assert!(
        HxCurrentUrl::from_request_parts(&mut parts, &())
            .await
            .unwrap()
            .0
            .is_none()
    );
}

#[test]
fn replace_url_sets_only_its_header_and_rejects_header_injection() {
    for url in ["?q=tea", "/todos", "false"] {
        let response = (HxReplaceUrl(url.into()), "ok").into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[HX_REPLACE_URL], url);
        assert!(!response.headers().contains_key(VARY));
    }
    let response = (HxReplaceUrl("/todos\r\nInjected: yes".into()), "ok").into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!response.headers().contains_key(HX_REPLACE_URL));
    assert!(!response.headers().contains_key("injected"));
}
