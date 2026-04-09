# Current Context

## Status

- The repository is initialized with a project README and an architecture document.
- A repo-specific `AGENTS.md` and lightweight `context/` system are in place.
- A Cargo workspace now exists with `ssw-core`, `ssw-actix`, `ssw-html`, `ssw-html-macros`, and `ssw-components`.
- The workspace compiles and passes `cargo test`.
- `ssw-html` now exposes an initial `html!` macro with a Maud-like authoring style.
- `ssw-html` now supports empty tags, `#id` and `.class` shorthand, optional attribute omission via `Option<T>`, and a small `Document` builder for page layouts.

## Current Priorities

- Keep the architecture tight around server-side rendering and server-owned state.
- Start with Actix, but avoid coupling the core to Actix-specific concerns.
- Refine the `ssw-html` authoring experience around real page and layout examples before committing to deeper renderer internals.
- Keep improving `ssw-html` around app-shaped use cases, not isolated rendering helpers.

## Open Questions

- Whether `ssw-core::Render` remains the right cross-crate rendering boundary once a richer HTML DSL exists.
- How much form and validation support should live in core versus adapter crates.
- Whether `ssw-html` should continue evolving its own macro parser or eventually absorb code from a Maud-derived implementation.
- How shorthand selectors and explicit attributes should compose without producing awkward rules.
- What the first end-to-end Actix example should prove beyond basic HTML responses.

## Next Likely Steps

- Expand `ssw-html` with more real-world ergonomics, such as attribute composition, reusable layout helpers, and clearer fragment helpers.
- Build a minimal end-to-end Actix example or test app that exercises the new `html!` authoring style.
- Revisit the `ssw-core` rendering boundary once `ssw-html` and Actix usage put more pressure on it.
