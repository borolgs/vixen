//! Embeds HTTP usage instructions in rendered pages.

use maud::{Markup, PreEscaped, html};

/// Generic instructions for driving a vixen app over HTTP.
///
/// The document includes Agent Skills frontmatter and can also be saved as a
/// `SKILL.md` file.
pub const SKILL: &str = include_str!("agent.md");

/// Renders [`SKILL`] and app-specific instructions in a non-executable script.
///
/// `app` should describe the application, authentication, and starting URL. It
/// must not contain `</script`.
///
/// ```
/// use vixen::{agent, maud::html};
///
/// let head = html! { (agent::skill("A todo list on one page. No login.")) }.into_string();
/// assert!(head.starts_with(r#"<script type="text/llms.txt">---"#));
/// assert!(head.contains("<hx-partial hx-target"));
/// assert!(head.ends_with("\n\nA todo list on one page. No login.</script>"));
/// ```
pub fn skill(app: impl AsRef<str>) -> Markup {
    html! {
        script type="text/llms.txt" {
            (PreEscaped(SKILL)) "\n" (PreEscaped(app.as_ref()))
        }
    }
}
