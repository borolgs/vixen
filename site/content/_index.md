+++
title = "vixen — server-driven UI with Rust, axum and htmx"
description = "Build server-driven UIs with axum and htmx through a concise, typed Rust API. Keep TypeScript and CSS beside your pages, with asset bundling built in."

[extra]
heading = "Rapid server-driven UI development with axum and htmx."
tagline = "A concise, typed Rust API with full access to htmx and modern frontend tools."
+++

Define an action with `#[action]`, call it from your markup, and return updated HTML fragments with `partial!`. vixen connects these pieces through Rust types, so you spend less time wiring up URLs and selectors.

## Try it out

{% include "demo.html" %}

## Client-side code belongs here, too.

Server-driven UI shouldn't mean managing frontend assets by hand. A page's TypeScript and CSS live beside its Rust code, in the same project.

```text
src/pages/dashboard/
├── mod.rs
├── index.ts
└── index.css
```

A chart or editor can use an npm library while the rest of the page stays server-driven.

vixen uses [Bun](https://bun.sh) to bundle each page's assets during the Rust build and embeds them in the application binary. Bun is needed to build the app, not to run it.

## Still axum. Still htmx.

Your handlers are still axum handlers, and htmx attributes are still available in your markup. vixen adds typed connections without replacing the tools underneath.

## Next steps

Start with the [counter](https://github.com/borolgs/vixen/tree/main/examples/counter), or explore [backlog](https://github.com/borolgs/vixen/tree/main/examples/backlog), a one-page issue tracker with inline editing and a sortable table.

[Installation and configuration](https://github.com/borolgs/vixen#installation) · [API reference](https://docs.rs/axum-vixen/latest/vixen/)
