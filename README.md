# ssw-rs

Server Side Web for Rust.

`ssw-rs` is an HTML-first framework for building durable web apps in Rust. It starts with server-side rendering, keeps application logic on the server, and treats JavaScript as an optional enhancement layer rather than the foundation of the app.

Plain strings passed into `ssw-html` document and component helpers are escaped by default. Use `Markup::raw(...)` only for trusted HTML.

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
- first-class remote and local font head helpers in `ssw-html::fonts`
- `.class` shorthand and composed `class=(...)` values
- optional attribute omission and HTML boolean-attribute semantics
- reusable form-field helpers in `ssw-components`
- redirect-carried flash messages in `ssw-core` and cookie-backed flash handling in `ssw-actix`
- cookie-backed CSRF hooks in `ssw-actix` with hidden form field helpers in `ssw-components`
- stable `ssw-*` classes and `data-*` state hooks on the current `ssw-components` primitives
- an optional first-party default theme stylesheet at `styles/ssw-theme-default.css`
- first layout and action primitives in `ssw-components`, including `button`, `container`, `section`, and `stack`
- first page-shell primitives in `ssw-components`, including `page_shell`, `page_header`, `page_actions`, and `card_header`
- first app-navigation and empty-state primitives in `ssw-components`, including `top_nav` and `empty_state`
- shared page-link and metadata primitives in `ssw-components`, including `link_button`, `MetaItem`, and `meta_list`
- a native `select` helper in `ssw-components`
- a workspace example app at `examples/ssw-intake-demo`
- a second workspace example app at `examples/ssw-projects-demo`
- an end-to-end Actix flow for page rendering, fragments, redirects, form mutation handling, field-level validation, flash messages, and CSRF verification

## Example

```rust
use ssw_html::{html, page};
use ssw_html::fonts;

let page = page("Dashboard")
    .body_class("app-shell")
    .head(fonts::local_font("Inter", "/static/fonts/Inter.var.woff2")
        .weight_range(100, 900)
        .preload())
    .body(html! {
        main #app .page {
            h1 { "Server Side Web" }
            p { "Rendered on the server." }
            button type="button" disabled=(false) { "Ready" }
        }
    })
    .render();
```

See [`ARCHITECTURE.md`](./ARCHITECTURE.md) for the v0 architecture and roadmap, and [`COMPONENTS.md`](./COMPONENTS.md) for the `ssw-components` design and theme split.

Run the current example app with:

```bash
cargo run -p ssw-intake-demo
```

Then open:

- `http://127.0.0.1:3000/` for the intake flow
- `http://127.0.0.1:3000/style-guide` for the live component preview

Capture fresh screenshots with:

```bash
./scripts/capture-intake-demo.sh
```

Or against a custom URL and output directory:

```bash
./scripts/capture-intake-demo.sh http://127.0.0.1:3001 ./artifacts/demo-pass
```

Run the second example app with:

```bash
cargo run -p ssw-projects-demo
```

The two examples now share page-level primitives instead of app-local link and metadata markup, which makes them a better pressure test for what should stay in `ssw-components`.

Useful capture knobs:

```bash
VIEWPORT_WIDTH=1720 VIEWPORT_HEIGHT=1320 ./scripts/capture-intake-demo.sh
FULL_PAGE=true ./scripts/capture-intake-demo.sh
```
