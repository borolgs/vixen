use vixen::action;

#[action("/save")]
struct Save {
    id: Option<i64>,
    name: String,
}

#[action("/search")]
struct Search {
    q: String,
    #[cursor]
    after: Option<i64>,
}

fn main() {
    let _: vixen::agent::Action = Save::skill();
    let _: String = vixen::skill![Save, Search,];
}
