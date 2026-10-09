use vixen::action;

#[action("/save")]
struct Save {
    #[serde(rename = "title")]
    name: String,
}

#[action("/search")]
#[serde(rename_all = "camelCase")]
struct Search {
    page_size: u32,
}

fn main() {}
