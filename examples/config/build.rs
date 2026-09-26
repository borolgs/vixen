fn main() {
    vixen::build(vixen::Config {
        base_path: "/config".into(),
        entry_glob: "src/index.ts".into(),
        ..Default::default()
    });
}
