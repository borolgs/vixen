fn main() {
    vixen::bundler::build(&vixen::bundler::Config {
        entry_glob: "src/index.ts".into(),
        ..Default::default()
    });
}
