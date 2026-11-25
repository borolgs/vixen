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

## Recording `static/demo.gif`

Playwright at <http://127.0.0.1:4007/>, Animate on. Inject overrides into the
recording page only; don't edit production CSS.

- Viewport: 836×470, DPR 2, light mode, no reduced motion. Don't hide the tabs.
  Clip `{ x: 0, y: 83, width: 836, height: 347 }` → 1672×694 without tabs.
- Temporary CSS (hide the Animate control without unchecking it):
  ```css
  .vixen-preview .preview-tabs { margin-bottom: 15px; }
  .vixen-preview .flow-animation { display: none !important; }
  .vixen-preview .sh__line::before { width: 1rem; }
  ```
  Load SF Mono from macOS Terminal's font bundle to avoid Courier. After the
  font loads, switch counter → hello to reset the source position.
- Scenario: hold 1 s, type `Casimir` (140 ms/character), wait 500 ms.
  Scope form controls to `[data-preview-example="hello"] .preview-demo`:
  the code bookmarks also have a button named Greet.
  Greet: `click({ delay: 220 })` captures its dark pressed state; move mouse away.
  Wait for the demo's `aria-busy="false"` and `!demo.inert`, then hold 2 s.
- Capture PNGs about every 40 ms with actual timestamps, caret hidden, animations allowed.
  In ffconcat, set `option framerate 1000` per file so timing isn't rounded to 25 fps.
- Transition: pause the 140 ms animation on `.preview-source pre` at time 0.
  Capture at `currentTime = 140 * i / 16`, i=0…15; play these frames 20 ms apart.
  Call `finish()`; subtract `(capture wall time - 320 ms)` from subsequent timestamps.
- Encode with full FFmpeg: 50 fps, `palettegen=stats_mode=full`,
  `paletteuse=dither=none:diff_mode=rectangle`, infinite loop (`-loop 0`).
  Set `-t` to the captured duration so the final repeated concat frame doesn't
  extend the loop.

Check the GIF: dark press ≈200 ms, 16 distinct transition frames, no tabs or
Animate control; 1672×694, ≈8–9 s / 600 KiB.
