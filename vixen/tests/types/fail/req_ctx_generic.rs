use vixen::ReqCtx;

#[derive(Clone, ReqCtx)]
struct Ctx<T> {
    value: T,
}

fn main() {}
