use vixen::{
    fragment,
    maud::{Markup, html},
};

use super::greet::Greet;

#[fragment]
pub fn form() -> Markup {
    html! {
        form id=(Self) hx-action=(Greet::action()) {
            input name=(Greet::FIELD.name) placeholder="Your name"
                aria-label="Your name" required maxlength="80";
            button type="submit" { "Greet" }
        }
    }
}
