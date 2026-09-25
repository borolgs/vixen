# components

Basecoat widgets driven by vixen actions and partial responses.

```bash
bun install               # once, for the frontend dependencies
cargo run -p components   # http://127.0.0.1:4003/
```

## What it shows

- The `basecoatui` feature provides `Toaster`, `Drawer`, `Dialog`, and `HEAD`.
  `HEAD` goes before the page bundle, while each widget renders one shell in
  the body.
- `show_toast` returns field fragments and a toast in one `HxPartial` response.
  The toast is a `Part` configured to append itself to the toaster.
- `open_drawer` and `confirm_delete` fill named header, content, and footer
  slots. htmx opens the corresponding shell after swapping those parts.
- `save_profile` uses `#[fragment]` for inline validation. On success it sends
  the drawer's close event with `HxResponseTrigger` and returns a toast as the
  response body.
- `build.ts` enables Tailwind. `app.css` imports Tailwind and Basecoat, while
  `app.ts` imports htmx and the Basecoat scripts used on the page.
