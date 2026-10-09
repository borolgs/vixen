# shop

A catalog with product and material admin pages, backed by in-memory SQLite.

```bash
bun install         # once, for the frontend dependencies
cargo run -p shop   # http://127.0.0.1:4005/
```

## Highlights

- The catalog grid and admin tables use `Paged`, `After` and `Page::from_rows`
  for keyset pagination. The admin tables use `Paged::table`, and
  `Paged::replace_url` adds the search parameters to the URL.
- Admin rows are dynamic `#[fragment]`s, so creates, updates and deletes can
  refresh or replace only the affected content.
- The product form has a client-filtered category `Combobox` and a
  server-searched, paginated material picker.
- Forms open in a `Drawer`; delete confirmations open in a `Dialog`.
- `Ctx` derives `ReqCtx`, giving handlers request-local database access without
  a `State` parameter.
- The catalog test uses `Ctx::scope` and `vixen::testing` to exercise a handler
  without starting a server.
