//! HTTP instructions for agents, embedded in pages or served at `/llms.txt`.
//!
//! Available with the `agent` feature.

use std::fmt;
use std::marker::PhantomData;

use axum::{Router, http::header::CONTENT_TYPE, routing::get};
use maud::{Markup, PreEscaped, html};
use serde::de::DeserializeOwned;

/// Shared HTTP instructions with Agent Skills frontmatter.
///
/// Save as `SKILL.md`, or use [`Skill`] to add app-specific instructions and actions.
pub const SKILL: &str = include_str!("agent.md");

/// App-specific instructions and an action index, appended to [`SKILL`].
///
/// Use [`text`](Self::text) for a `SKILL.md` file, [`script`](Self::script) for
/// the page head, or [`(router)`](Self::router) to serve `/llms.txt`.
///
/// ```
/// use vixen::agent::Skill;
///
/// let skill = Skill::new("A todo list at GET /, settings at GET /settings. No login.")
///     .name("todos")
///     .description("Drive the todos app over HTTP with curl.")
///     .index("POST /todos/add {title: String}")
///     .index("POST /settings/theme {theme: light | dark}");
///
/// // Every page's head carries it; the router serves it at GET /llms.txt.
/// let script = skill.script();
/// let router: axum::Router = skill.router();
/// ```
pub struct Skill {
    app: String,
    name: Option<String>,
    description: Option<String>,
    index: String,
    rewrite: Option<Box<dyn Fn(String) -> String + Send + Sync>>,
}

impl Skill {
    /// Creates a skill with an app note: its purpose, starting URL and login requirements.
    pub fn new(app: impl Into<String>) -> Self {
        Skill {
            app: app.into(),
            name: None,
            description: None,
            index: String::new(),
            rewrite: None,
        }
    }

    /// Sets the frontmatter `name`. Use the app's name for a saved skill.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the frontmatter `description`. Use app-specific wording for a saved skill.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds action lines, usually from [`skill!`](crate::skill!); one call per page.
    pub fn index(mut self, index: impl Into<String>) -> Self {
        let index = index.into();
        if index.is_empty() {
            return self;
        }
        if !self.index.is_empty() {
            self.index.push('\n');
        }
        self.index.push_str(&index);
        self
    }

    /// Edits the assembled text last; `text`, `script` and `router` all return the result.
    pub fn rewrite(mut self, f: impl Fn(String) -> String + Send + Sync + 'static) -> Self {
        self.rewrite = Some(Box::new(f));
        self
    }

    /// Returns the full skill text: frontmatter, HTTP instructions, app note and action index.
    pub fn text(&self) -> String {
        let (frontmatter, body) = SKILL
            .strip_prefix("---\n")
            .and_then(|skill| skill.split_once("\n---\n"))
            .expect("agent.md frontmatter");
        let mut text = String::from("---\n");
        for line in frontmatter.lines() {
            let (key, value) = line.split_once(": ").expect("agent.md frontmatter");
            let value = match key {
                "name" => self.name.as_deref().unwrap_or(value),
                "description" => self.description.as_deref().unwrap_or(value),
                _ => value,
            };
            text.push_str(&format!("{key}: {value}\n"));
        }
        text.push_str("---\n");
        text.push_str(body);
        text.push('\n');
        text.push_str(&self.app);
        if !self.index.is_empty() {
            text.push_str("\n\n");
            text.push_str(&self.index);
        }
        match &self.rewrite {
            Some(rewrite) => rewrite(text),
            None => text,
        }
    }

    /// Embeds the skill in a non-executable `text/llms.txt` script for the page head.
    ///
    /// The text is not HTML-escaped and must not contain `</script`.
    pub fn script(&self) -> Markup {
        html! { script type="text/llms.txt" { (PreEscaped(self.text())) } }
    }

    /// Returns a router serving the skill as plain text at `GET /llms.txt`.
    pub fn router<S: Clone + Send + Sync + 'static>(&self) -> Router<S> {
        let text = self.text();
        Router::new().route(
            "/llms.txt",
            get(move || {
                let text = text.clone();
                async move { ([(CONTENT_TYPE, "text/plain; charset=utf-8")], text) }
            }),
        )
    }
}

/// Route and form fields for one entry in the action index.
///
/// `#[action]` generates this through `Name::skill()`. For other POST routes,
/// construct it directly.
///
/// ```
/// use vixen::agent::{Action, Field};
///
/// let action = Action {
///     base: "/config",
///     path: "/theme",
///     fields: vec![
///         Field { name: "theme", ty: "Theme", values: Some(&["light", "dark"]), optional: false, repeated: false },
///         Field { name: "remember", ty: "bool", values: None, optional: true, repeated: false },
///     ],
///     returns: Some("The page in the new theme."),
/// };
/// assert_eq!(
///     action.to_string(),
///     "POST /config/theme {theme: light | dark, remember?: bool}  The page in the new theme."
/// );
/// ```
pub struct Action {
    /// The app's base path, `""` when there is none.
    pub base: &'static str,
    /// The route, as declared.
    pub path: &'static str,
    /// The form fields, in declaration order.
    pub fields: Vec<Field>,
    /// What the action returns: the first line of the struct's doc comment.
    pub returns: Option<&'static str>,
}

/// One form field of an [`Action`].
pub struct Field {
    /// The form key.
    pub name: &'static str,
    /// Rust type name without `Option<T>` or `Vec<T>` wrappers; `cursor` for `#[cursor]`.
    pub ty: &'static str,
    /// Allowed values from [`strum::VariantNames`], displayed instead of [`ty`](Self::ty).
    pub values: Option<&'static [&'static str]>,
    /// Whether the field may be omitted (`Option<T>` or `#[serde(default)]`).
    pub optional: bool,
    /// Whether to send multiple values by repeating the form key (`Vec<T>`).
    pub repeated: bool,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "POST {}{} {{", self.base, self.path)?;
        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            fmt::Display::fmt(field, f)?;
        }
        f.write_str("}")?;
        match self.returns {
            Some(returns) => write!(f, "  {returns}"),
            None => Ok(()),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let optional = if self.optional { "?" } else { "" };
        write!(f, "{}{optional}: ", self.name)?;
        match (self.values, self.repeated) {
            (Some(values), true) => write!(f, "({})[]", values.join(" | ")),
            (Some(values), false) => f.write_str(&values.join(" | ")),
            (None, true) => write!(f, "{}[]", self.ty),
            (None, false) => f.write_str(self.ty),
        }
    }
}

/// Formats actions as newline-separated index entries, without a trailing newline.
///
/// ```
/// use vixen::agent::{Action, index};
///
/// let ping = Action { base: "", path: "/ping", fields: vec![], returns: None };
/// let pong = Action { base: "", path: "/pong", fields: vec![], returns: None };
/// assert_eq!(index(&[ping, pong]), "POST /ping {}\nPOST /pong {}");
/// ```
pub fn index(actions: &[Action]) -> String {
    actions
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Builds an action index from `#[action]` types for [`Skill::index`](crate::agent::Skill::index).
///
/// Each line has the form `POST /path {field: Type, ..}`, followed by the first line of
/// the struct's doc comment: what the action returns. Field notation:
///
/// - `field?`: optional (`Option<T>` or `#[serde(default)]`).
/// - `Type[]`: repeated form key (`Vec<T>`).
/// - `a | b`: enum values from [`strum::VariantNames`], instead of the type name.
/// - `cursor`: a `#[cursor]` field for pagination.
///
/// Debug builds panic when serde rejects a listed value, so `#[strum(serialize_all)]`
/// must match `#[serde(rename_all)]`.
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use vixen::action;
///
/// #[derive(Deserialize, Serialize, strum::VariantNames)]
/// #[serde(rename_all = "lowercase")]
/// #[strum(serialize_all = "lowercase")]
/// enum Status {
///     Open,
///     Done,
/// }
///
/// /// The issue's row.
/// #[action("/issues/status")]
/// struct IssueStatus {
///     id: i64,
///     status: Status,
/// }
///
/// #[action("/issues/move")]
/// struct IssuesMove {
///     #[serde(default)]
///     ids: Vec<i64>,
///     status: Option<Status>,
/// }
///
/// #[action("/issues/list")]
/// struct IssueList {
///     #[cursor]
///     after: Option<i64>,
/// }
///
/// assert_eq!(
///     vixen::skill![IssueStatus, IssuesMove, IssueList],
///     concat!(
///         "POST /issues/status {id: i64, status: open | done}  The issue's row.\n",
///         "POST /issues/move {ids?: i64[], status?: open | done}\n",
///         "POST /issues/list {after?: cursor}",
///     )
/// );
/// ```
#[macro_export]
macro_rules! skill {
    ($($action:ty),* $(,)?) => {
        $crate::agent::index(&[$(<$action>::skill()),*])
    };
}

// Autoref fallback: `&Probe<T>` selects `Values` when `T: VariantNames`;
// otherwise method lookup adds a reference and selects `NoValues`.
#[doc(hidden)]
pub struct Probe<T>(PhantomData<T>);

impl<T> Probe<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Probe(PhantomData)
    }
}

#[doc(hidden)]
pub trait Values {
    fn values(&self) -> Option<&'static [&'static str]>;
}

impl<T: strum::VariantNames + DeserializeOwned> Values for Probe<T> {
    fn values(&self) -> Option<&'static [&'static str]> {
        // The index must list what the form accepts; strum and serde name variants separately.
        #[cfg(debug_assertions)]
        {
            #[derive(serde::Deserialize)]
            struct Form<T> {
                v: T,
            }
            for variant in T::VARIANTS {
                let query = serde_html_form::to_string([("v", *variant)]).expect("a form pair");
                let form = serde_html_form::from_str::<Form<T>>(&query);
                if let Err(err) = form.map(|form| form.v) {
                    panic!(
                        "`{}::VARIANTS` lists `{variant}` but the form rejects it: {err}",
                        std::any::type_name::<T>()
                    );
                }
            }
        }
        Some(T::VARIANTS)
    }
}

#[doc(hidden)]
pub trait NoValues {
    fn values(&self) -> Option<&'static [&'static str]>;
}

impl<T> NoValues for &Probe<T> {
    fn values(&self) -> Option<&'static [&'static str]> {
        None
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn text_swaps_the_frontmatter_and_appends_the_app_note_and_index() {
        let app = "A todo list on one page at GET /. No login.";
        let body = SKILL.split_once("\n---\n").unwrap().1;
        let text = Skill::new(app)
            .name("todos")
            .description("Drive the todos app over HTTP with curl.")
            .index("POST /todos/add {title: String}")
            .index("POST /settings/theme {theme: light | dark}")
            .text();
        assert_eq!(
            text,
            format!(
                "---\nname: todos\ndescription: Drive the todos app over HTTP with curl.\n---\n\
                 {body}\n{app}\n\nPOST /todos/add {{title: String}}\nPOST /settings/theme {{theme: light | dark}}"
            )
        );

        assert_eq!(Skill::new(app).text(), format!("{SKILL}\n{app}"));
    }

    #[test]
    fn rewrite_runs_last_and_the_script_embeds_the_result() {
        let skill = Skill::new("A todo list on one page at GET /. No login.")
            .index("POST /todos/add {title: String}")
            .rewrite(|text| text + "\n\n## Examples");
        let text = skill.text();
        assert!(text.ends_with("POST /todos/add {title: String}\n\n## Examples"));
        assert_eq!(
            skill.script().into_string(),
            format!(r#"<script type="text/llms.txt">{text}</script>"#)
        );
    }

    #[test]
    fn returns_is_the_first_doc_line() {
        /** The moved rows.

        Posted by the bulk bar. */
        #[vixen::action("/issues/move")]
        struct IssuesMove {}

        assert_eq!(
            IssuesMove::skill().to_string(),
            "POST /issues/move {}  The moved rows."
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    fn values_check_through_the_form_parser() {
        #[derive(serde::Deserialize, strum::VariantNames)]
        #[serde(try_from = "u8")]
        enum Priority {
            #[strum(serialize = "1")]
            Low,
        }

        impl TryFrom<u8> for Priority {
            type Error = String;
            fn try_from(n: u8) -> Result<Self, String> {
                (n == 1)
                    .then_some(Priority::Low)
                    .ok_or(format!("no priority {n}"))
            }
        }

        assert_eq!(Probe::<Priority>::new().values(), Some(&["1"][..]));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(
        expected = "Status::VARIANTS` lists `in_progress` but the form rejects it: unknown variant `in_progress`"
    )]
    fn values_panic_when_the_form_rejects_a_listed_variant() {
        #[derive(serde::Deserialize, strum::VariantNames)]
        #[serde(rename_all = "kebab-case")]
        #[strum(serialize_all = "snake_case")]
        enum Status {
            Backlog,
            InProgress,
        }

        let _ = Probe::<Status>::new().values();
    }

    #[tokio::test]
    async fn serves_the_text_at_llms_txt() {
        let skill = Skill::new("A todo list on one page at GET /. No login.")
            .index("POST /todos/add {title: String}")
            .rewrite(|text| text + "\n\n## Examples");
        let router: Router = skill.router();
        let req = Request::get("/llms.txt").body(Body::empty()).unwrap();
        let res = router.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[CONTENT_TYPE], "text/plain; charset=utf-8");
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], skill.text().as_bytes());
    }
}
