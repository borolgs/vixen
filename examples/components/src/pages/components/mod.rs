use axum::{Router, response::IntoResponse};
use vixen::{
    HxPartial, action, fragment,
    hx::SwapOption,
    maud::{Markup, html},
    routing::RouterExt,
    ui::basecoatui::{Action, Category, Duration, HEAD, Toast, Toaster},
    view_path,
};

use crate::shared::page;

pub fn router() -> Router {
    Router::new().typed_get(home).typed_post(show_toast)
}

#[view_path("/")]
struct HomePath;

async fn home(_: HomePath) -> Markup {
    page(
        html! {
            title { "Components · vixen" }
            (HEAD)
            (vixen::assets!())
        },
        html! {
            header.mb-8 {
                h1.text-2xl.font-semibold.tracking-tight { "Components" }
                p.text-muted-foreground { "Basecoat widgets wired through vixen actions." }
            }
            main.space-y-6 {
                section.card {
                    header {
                        h2 { "Toaster" }
                        p { "The action returns a partial appended to the toaster." }
                        a class="text-sm text-muted-foreground hover:underline"
                            href="https://basecoatui.com/components/toast/" target="_blank" rel="noreferrer" {
                            "basecoatui.com/components/toast ↗"
                        }
                    }
                    section {
                        (toast_form())
                    }
                }
            }
            (Toaster::new().shell())
        },
    )
}

// Toaster

const TOASTER: Toaster = Toaster::new();

#[action("/toast")]
struct ShowToast {
    message: String,
    category: Category,
    duration: Option<u32>,
}

async fn show_toast(
    ShowToast {
        message,
        category,
        duration,
    }: ShowToast,
) -> impl IntoResponse {
    HxPartial::new()
        .part(message_input())
        .part(duration_input())
        .part(TOASTER.toast(Toast {
            category,
            title: message,
            description: None,
            duration: duration.map(Duration::Millis).unwrap_or_default(),
            action: Some(Action::Dismiss("Dismiss".into())),
        }))
}

#[fragment]
fn toast_form() -> Markup {
    const CATEGORIES: [(&str, &str); 4] = [
        ("success", "Success"),
        ("info", "Info"),
        ("warning", "Warning"),
        ("error", "Error"),
    ];

    html! {
        form.grid.gap-6 id=(Self) hx-action=(ShowToast::action().hx().swap(SwapOption::OuterHtml))
        {
            div class="flex gap-2" {
                (message_input())
                (duration_input::slot())
            }
            fieldset.fieldset {
                legend data-variant="label" { "Category" }
                div role="radiogroup" aria-label="Category" class="pb-1" {
                    @for (value, label) in CATEGORIES {
                        div.field role="group" data-orientation="horizontal" {
                            input.input id={ "toast-" (value) } hx-preserve="true" type="radio" name=(ShowToast::FIELD.category)
                                value=(value) checked[value == "success"];
                            label.font-normal for={ "toast-" (value) } { (label) }
                        }
                    }
                }
            }
            div { button.btn type="submit" { "Show a toast" } }
        }
    }
}

#[fragment]
fn message_input() -> Markup {
    html! {
        div.field id=(Self) role="group" {
            label for="toast-message" { "Message" }
            input.input id="toast-message" type="text" name=(ShowToast::FIELD.message)
                placeholder="It is on the shelf now." required;
        }
    }
}

#[fragment]
/// Optional duration input
fn duration_input() -> Markup {
    html! {
        div.field id=(Self) role="group" {
            label for="toast-duration" { "Duration" }
            input.input id="toast-duration" hx-preserve="true" type="text" name=(ShowToast::FIELD.duration)
                placeholder="You can set a duration too.";
        }
    }
}
