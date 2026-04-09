# ssw-rs

Server Side Web for Rust.

`ssw-rs` is an HTML-first framework for building durable web apps in Rust. It starts with server-side rendering, keeps application logic on the server, and treats JavaScript as an optional enhancement layer rather than the foundation of the app.

## Goals

- Fast and predictable server-rendered web apps
- Strong Rust ergonomics for authoring HTML
- Minimal dependencies and explicit abstractions
- First-class forms, redirects, validation, and partial rendering
- Optional progressive enhancement without forcing client-side routing

## Current slice

The current implementation targets Actix, with crate boundaries designed so additional backends can be added later.

Workspace crates:

- `ssw-core`
- `ssw-actix`
- `ssw-html`
- `ssw-components`

Internal support crate:

- `ssw-html-macros`

Currently implemented:

- `ssw-html::html!` with Maud-like syntax
- document and fragment rendering
- `.class` shorthand and composed `class=(...)` values
- optional attribute omission and HTML boolean-attribute semantics
- reusable form-field helpers in `ssw-components`
- an end-to-end Actix flow for page rendering, fragments, redirects, form mutation handling, and field-level validation

## Example

```rust
use ssw_html::{html, page};

let page = page("Dashboard")
    .body_class("app-shell")
    .body(html! {
        main #app .page {
            h1 { "Server Side Web" }
            p { "Rendered on the server." }
            button type="button" disabled=(false) { "Ready" }
        }
    })
    .render();
```

See [`ARCHITECTURE.md`](./ARCHITECTURE.md) for the v0 architecture and roadmap.
