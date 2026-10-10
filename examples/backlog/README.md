# backlog

A one-page issue tracker with inline editing, backed by in-memory SQLite.

```bash
bun install            # once, for the frontend dependencies
cargo run -p backlog   # http://127.0.0.1:4006/
```

## What it shows

- Each editable cell posts on `change`. Its action returns the updated row and
  stats; validation errors restore the saved row and show a toast.
- `ISSUES` uses `Paged::table` with sort-aware keyset pagination. Its `After`
  cursor contains the sort key and issue id. `SortHead` in `src/sort.rs`
  renders column headers that re-sort through the search form.
- `Paged::replace_url` keeps the search, status filter, and sort in the URL.
  `issue_title`, `issue_status` and `issues_move` use `HxCurrentQuery` to remove
  rows that no longer match the current filter.
- `issue_add` prepends a row with `SwapOption::AfterBegin`; it does not reload
  the list.
- Row checkboxes join the bulk-action form through the `form` attribute. CSS
  shows the bulk bar when rows are selected. `page.ts` handles select-all and
  restores an edited cell when Escape is pressed.
- An edited row stays in place when its sort key changes. `page.ts` drops that
  copy when a later page returns the row.
- Deleting rows assigns them a batch id. The toast's Undo action passes that id
  to `issues_restore`, which refreshes the list with `ISSUES.refresh()`.
