use vixen::id;

#[id]
struct TodoId {
    id: u64,
}

#[id]
struct CellId(u32, u32);

fn main() {}
