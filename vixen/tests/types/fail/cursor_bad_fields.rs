use vixen::action;

#[action("/search")]
struct Search {
    #[cursor]
    after: u32,
}

#[action("/browse")]
struct Browse {
    #[cursor]
    after: Option<u32>,
    #[cursor]
    before: Option<u32>,
}

fn main() {}
