# Site

Requires Rust and Bun. Run from the repository root:

```bash
# One-time setup; versions match .github/workflows/pages.yml
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.1 --locked
cargo install --locked --git https://github.com/getzola/zola --tag v0.23.6
bun install --frozen-lockfile

# Build and preview
cd site
./build.sh
zola serve
```

Output: `site/dist/`. Preview: <http://127.0.0.1:1111/>.
After WASM changes, stop the server, rerun `./build.sh`, and restart `zola serve`.
