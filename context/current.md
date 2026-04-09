# Current Context

## Status

- The repository is initialized with a project README and an architecture document.
- A repo-specific `AGENTS.md` and lightweight `context/` system are in place.
- A Cargo workspace now exists with `ssw-core`, `ssw-actix`, `ssw-html`, and `ssw-components`.
- The workspace compiles and passes `cargo test`.

## Current Priorities

- Keep the architecture tight around server-side rendering and server-owned state.
- Start with Actix, but avoid coupling the core to Actix-specific concerns.
- Keep the current HTML layer intentionally small until the Maud wrapper versus fork decision is grounded in real API pressure.

## Open Questions

- Whether `ssw-html` should begin as a wrapper around Maud or a minimal maintained fork.
- Whether `ssw-core::Render` remains the right cross-crate rendering boundary once a richer HTML DSL exists.
- How much form and validation support should live in core versus adapter crates.
- What the first end-to-end Actix example should prove beyond basic HTML responses.

## Next Likely Steps

- Build a minimal end-to-end Actix example or test app.
- Expand `ssw-core` with clearer response helpers for pages, fragments, and redirects.
- Decide whether to evolve `ssw-html` in place or replace it with a Maud-derived implementation.
