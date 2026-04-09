# ssw-rs

Server Side Web for Rust.

`ssw-rs` is an HTML-first framework for building durable web apps in Rust. It starts with server-side rendering, keeps application logic on the server, and treats JavaScript as an optional enhancement layer rather than the foundation of the app.

## Goals

- Fast and predictable server-rendered web apps
- Strong Rust ergonomics for authoring HTML
- Minimal dependencies and explicit abstractions
- First-class forms, redirects, validation, and partial rendering
- Optional progressive enhancement without forcing client-side routing

## Initial shape

The initial implementation targets Actix, with crate boundaries designed so additional backends can be added later.

Planned crates:

- `ssw-core`
- `ssw-actix`
- `ssw-html`
- `ssw-components`

See [`ARCHITECTURE.md`](./ARCHITECTURE.md) for the v0 architecture and roadmap.
