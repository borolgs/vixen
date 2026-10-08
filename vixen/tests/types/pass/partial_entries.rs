use vixen::{
    HxPartial, HxPartialResponse, Part, Parts,
    hx::{HxEvent, SwapOption},
    maud::{Markup, html},
    partial,
};

fn toast() -> Part {
    Part::new("#toaster", html! { "saved" }).swap(SwapOption::BeforeEnd)
}

fn rows() -> Markup {
    html! { tr { td { "row" } } }
}

fn both() -> Parts {
    vec![Part::new("#rows", rows()), toast()].into()
}

fn main() {
    // A bare part before a targeted entry: the `target => content` arm must
    // fail over to the bare arm on the comma, not eat the part as a target.
    let _ = partial!(toast(), "#rows" => rows());
    let _ = partial!("#rows" => rows(), toast());
    let _ = partial!(toast());
    let _ = partial!(toast(), toast(),);
    let _ = partial!(_ => rows(), toast(), "#rows" => rows());
    let _: HxPartial<_, _> = partial!(_ => rows());

    let _ = partial!(both());
    let _ = partial!(vec![toast(), toast()]);
    let _ = partial!(_ => rows(), both(), "#rows" => rows());

    // `(target, swap) => content`; a parenthesised bare part is still a part.
    let _ = partial!(("#rows", SwapOption::AfterEnd) => rows(), toast());
    let _ = partial!(toast(), ("#rows", SwapOption::AfterEnd) => rows());
    let _ = partial!((toast()), "#rows" => rows());

    // Events and `None` do not count as response content.
    let _: HxPartialResponse = partial!(HxEvent::new("saved"), toast(), None::<Part>);
    let _ = partial!(_ => rows(), Some(toast()), HxEvent::new("saved"));
    let _: HxPartialResponse = HxPartial::new().event("saved").entry(toast());
}
