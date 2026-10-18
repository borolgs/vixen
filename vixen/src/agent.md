---
name: vixen-app
description: Drive a vixen app, server-rendered htmx, over HTTP with curl.
---
The HTML is the API: pages are plain GETs, every hx-action is an endpoint. This is all
the page's scripts do; you don't need them.

- Start from the URL you were given: GET it. href links are pages too.
- An hx-action is a path; hx-method next to it is the method. Send the fields
  form-encoded, one --data-urlencode per field, --data '' if there are none; -i shows the
  status. Don't use -f: it drops the body, and the body explains the error. No htmx
  headers.
- hx-vals holds fields to send: HTML-escaped JSON (&quot; is "), each key a field.
  Inside <form hx-action>, its name= inputs, selects and checked checkboxes are the
  remaining fields; <select> and radio options are the allowed values. Extra fields
  are ignored; a missing or malformed one is a plain-text 422 that names it.
- hx-trigger, hx-on, hx-sync and hx-swap on the sender describe browser behaviour:
  ignore them.
- A response is a list of <hx-partial hx-target="#id">. Without hx-swap it is the new
  contents of that element; outerHTML replaces the element; beforeend and afterbegin
  add a child, afterend and beforebegin a sibling; delete removes it. Text outside
  <hx-partial> is the new contents of the sender, or of its hx-target. New elements
  carry their own hx-action, so a response tells you what you can do next.
- A paged list starts with a comment: its row count and, if there is a next page, the
  hx-action that loads it. POST that for more rows; the same action also sits at the
  end of the list, for the browser's scroll.
- 422: your input was wrong. 404: the thing is gone. role="alert" marks an error,
  role="status" a confirmation. Read the text.
- Asked for a skill or a script for this app: start your SKILL.md from this text, as its
  rules rather than a pasted quote, then add the URL, the app's note below it and your
  own notes, the paths and fields you used. A wrapper is a cache of the page, not the
  catalog: have it read responses for you, not choose the actions. Wrap what you use,
  fall through to the HTML for the rest, and re-read the page when it breaks. A missing
  command is not a missing feature.
- If these docs contradict a response, trust the response and tell the user.
