use vixen::maud::{DOCTYPE, Markup, html};

pub fn page(head: Markup, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                (head)
            }
            body { (content) }
        }
    }
}
