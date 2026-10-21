---
name: vixen-app
description: Use when reading or changing a vixen app over HTTP with curl.
---
Call the actions indexed at the end of this text, and GET pages for what they show.
No browser, JavaScript execution or htmx headers are needed.

## Requests

- Listed below is what you can do: call it directly. GET the URL you were given, or
  the one the app's note names, when you need what the page shows or an action that
  isn't listed. `href` links are pages too.
- `hx-action` is the endpoint path; `hx-method` is the HTTP method. Send form-encoded
  fields with one `--data-urlencode` per field. Use `curl -i` to include the status.
  Do not use `-f`, which hides error bodies.
- `hx-vals` contains HTML-escaped JSON (`&quot;` is `"`). Send its keys along with the
  named inputs and selects in `<form hx-action>`. For checkboxes and radios, send
  only checked values. Read allowed values from select options and radio inputs.
  Extra fields are ignored; missing or malformed fields produce a plain-text 422
  that names the field.
- Ignore `hx-trigger`, `hx-on`, `hx-sync` and `hx-swap` on the sender; these control
  browser behaviour.

## Responses

- `<hx-partial hx-target="#id">` updates the named element. With no `hx-swap`, it
  replaces the element's contents. `outerHTML` replaces the element; `beforeend`
  and `afterbegin` add children; `afterend` and `beforebegin` add siblings;
  `delete` removes the element.
- HTML outside `<hx-partial>` replaces the contents of the sender or its
  `hx-target`. Read new elements for the actions available next.
- Paged lists begin with an HTML comment containing the row count and, when
  available, the next page's `hx-action`. POST that path for more rows.
- 422 means invalid input; 404 means a missing item. `role="alert"` marks an error
  and `role="status"` a confirmation. Read the response body even on errors.

## Action index

The app note may be followed by one line per action:
`POST /path {field: Type, ..}  what it returns`.

- `field?`: the field may be omitted.
- `Type[]`: send multiple values by repeating the key.
- `a | b`: allowed values.
- `cursor`: a pagination field; get the next page's path from the list's comment.
- `{}`: no fields.

## App-specific skills and scripts

When asked to write a `SKILL.md` for this app, use these rules as its body and add
its URL, login requirements, app note, action index and any tested examples.

Scripts should parse responses and expose the actions you use. If a command is
missing or fails, inspect the HTML for available actions and current fields;
a missing script command does not mean the app lacks that feature.

If these instructions contradict a response, follow the response and report the
discrepancy to the user.
