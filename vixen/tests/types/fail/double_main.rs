use maud::html;
use vixen::HxPartial;

fn main() {
    let _ = HxPartial::new().main(html! { "one" }).main(html! { "two" });
}
