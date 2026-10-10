use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use vixen::{
    HxAction, Id, SyncStrategy,
    hx::SwapOption,
    maud::{Markup, Render, html},
};

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, Debug)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SortDir {
    Asc,
    #[default]
    Desc,
}

impl SortDir {
    pub fn toggle(self) -> Self {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }

    /// The keyset operator and the `order by` direction.
    pub fn sql(self) -> (&'static str, &'static str) {
        match self {
            SortDir::Asc => (">", "asc"),
            SortDir::Desc => ("<", "desc"),
        }
    }
}

/// Column headers that re-sort a `Paged` list through its search form.
pub struct SortHead<F, I> {
    list: &'static str,
    form: I,
    sort: F,
    dir: SortDir,
    resort: fn(F, SortDir) -> HxAction,
}

impl<F: Copy + PartialEq + AsRef<str>, I: Id + Render> SortHead<F, I> {
    pub fn new(
        list: &'static str,
        form: I,
        sort: F,
        dir: SortDir,
        resort: fn(F, SortDir) -> HxAction,
    ) -> Self {
        Self {
            list,
            form,
            sort,
            dir,
            resort,
        }
    }

    /// Keeps the current sort in the search form.
    pub fn state(&self) -> Markup {
        html! {
            input type="hidden" form=(self.form) name="sort" value=(self.sort.as_ref());
            input type="hidden" form=(self.form) name="dir" value=(self.dir.as_ref());
        }
    }

    pub fn button(&self, label: &str, field: F) -> Markup {
        let active = self.sort == field;
        let dir = if active {
            self.dir.toggle()
        } else {
            SortDir::Desc
        };
        let arrow = match (active, self.dir) {
            (false, _) => "",
            (true, SortDir::Asc) => "↑",
            (true, SortDir::Desc) => "↓",
        };

        let resort = (self.resort)(field, dir)
            .target(format!("#{}", self.list))
            .swap(SwapOption::OuterHtml)
            .sync(SyncStrategy::Replace.on(&self.form));

        html! {
            button.btn."-ml-3" type="button" data-variant="ghost" data-size="sm"
                form=(self.form) hx-action=(resort)
            {
                (label)
                span class="text-muted-foreground w-3" { (arrow) }
            }
        }
    }
}
