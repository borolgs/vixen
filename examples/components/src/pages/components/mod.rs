use axum::{Router, response::IntoResponse};
use vixen::{
    HxPartial, action, fragment,
    hx::{HxResponseTrigger, SwapOption},
    maud::{Markup, html},
    partial,
    routing::RouterExt,
    ui::basecoatui::{Action, Category, Dialog, Drawer, Duration, HEAD, Toast, Toaster},
    view_path,
};

use crate::shared::page;

pub fn router() -> Router {
    Router::new()
        .typed_get(home)
        .typed_post(show_toast)
        .typed_post(open_drawer)
        .typed_post(save_profile)
        .typed_post(confirm_delete)
        .typed_post(delete_shelf)
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
                section.card {
                    header {
                        h2 { "Drawer" }
                        p { "An action fills the drawer's slots, then the swap opens it." }
                        a class="text-sm text-muted-foreground hover:underline"
                            href="https://basecoatui.com/components/drawer/" target="_blank" rel="noreferrer" {
                            "basecoatui.com/components/drawer ↗"
                        }
                    }
                    section {
                        button.btn data-variant="outline" type="button" hx-action=(OpenDrawer::action()) { "Edit profile" }
                    }
                }
                section.card {
                    header {
                        h2 { "Dialog" }
                        p { "One action opens the confirmation; another closes it." }
                        a class="text-sm text-muted-foreground hover:underline"
                            href="https://basecoatui.com/components/dialog/" target="_blank" rel="noreferrer" {
                            "basecoatui.com/components/dialog ↗"
                        }
                    }
                    section {
                        button.btn data-variant="destructive" type="button" hx-action=(ConfirmDelete::action()) { "Delete the shelf" }
                    }
                }
            }
            (TOASTER.shell())
            (DRAWER.shell())
            (CONFIRM.shell())
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
                legend data-variant="label" class="mb-2" { "Category" }
                div role="radiogroup" aria-label="Category" {
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

// Drawer

const DRAWER: Drawer = Drawer::new("profile").content_class("px-4");

#[action("/drawer")]
struct OpenDrawer {}

#[action("/drawer/save")]
struct SaveProfile {
    name: String,
}

async fn open_drawer(_: OpenDrawer) -> impl IntoResponse {
    DRAWER
        .header(html! {
            h2 { "Edit profile" }
            p { "Save closes the drawer; an empty name keeps it open." }
        })
        .content(html! {
            form.grid.gap-6 id="profile-form" hx-action=(SaveProfile::action()) {
                (profile_name_input("", None))
            }
        })
        .footer(html! {
            button.btn type="submit" form="profile-form" { "Save" }
            button.btn data-variant="outline" type="button" onclick="this.closest('dialog').close()" { "Cancel" }
        })
}

async fn save_profile(SaveProfile { name }: SaveProfile) -> impl IntoResponse {
    let name = name.trim();
    if name.is_empty() {
        return partial! {
            profile_name_input(name, Some("Enter a name."))
        }
        .into_response();
    }

    (
        HxResponseTrigger::normal([DRAWER.close()]),
        TOASTER.success("Saved", format!("Hello, {name}.")),
    )
        .into_response()
}

#[fragment]
fn profile_name_input(value: &str, error: Option<&str>) -> Markup {
    let id = SaveProfile::FIELD.name;
    let error_id = format!("{id}-error");
    html! {
        div.field id=(Self) role="group" data-invalid[error.is_some()] {
            label for=(id) { "Name" }
            input.input id=(id) type="text" name=(id) value=(value)
                aria-invalid=[error.map(|_| "true")]
                aria-describedby=[error.map(|_| error_id.as_str())];
            @if let Some(error) = error {
                p id=(error_id) role="alert" { (error) }
            }
        }
    }
}

// Dialog

const CONFIRM: Dialog = Dialog::new("confirm");

#[action("/dialog")]
struct ConfirmDelete {}

#[action("/dialog/delete")]
struct DeleteShelf {}

async fn confirm_delete(_: ConfirmDelete) -> impl IntoResponse {
    CONFIRM
        .header(html! {
            h2 { "Delete the shelf?" }
            p { "Everything on it goes too." }
        })
        .footer(html! {
            button.btn data-variant="outline" type="button" onclick="this.closest('dialog').close()" { "Cancel" }
            button.btn data-variant="destructive" type="button" hx-action=(DeleteShelf::action()) { "Delete" }
        })
}

async fn delete_shelf(_: DeleteShelf) -> impl IntoResponse {
    (
        HxResponseTrigger::normal([CONFIRM.close()]),
        TOASTER.success("Deleted", "The shelf is gone."),
    )
}
