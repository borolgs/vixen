fn main() {
    vixen::build(vixen::Config {
        entry_glob: "src/index.ts".into(),
        ..Default::default()
    });
}
