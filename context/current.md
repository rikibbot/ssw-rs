# Current Context

## Status

- The repository is initialized with a project README and an architecture document.
- A repo-specific `AGENTS.md` and lightweight `context/` system are in place.
- A Cargo workspace now exists with `ssw-core`, `ssw-actix`, `ssw-html`, `ssw-html-macros`, and `ssw-components`.
- The workspace compiles and passes `cargo test`.
- `ssw-html` now exposes an initial `html!` macro with a Maud-like authoring style.
- `ssw-html` now supports empty tags, `#id` and `.class` shorthand, optional attribute omission via `Option<T>`, and a small `Document` builder for page layouts.
- `ssw-html` now supports class composition between `.class` shorthand and explicit `class=(...)` values.
- `ssw-html` now renders expression-based boolean HTML attributes with presence/absence semantics instead of `="true"` or `="false"` strings.
- `ssw-html` has now been exercised through an end-to-end Actix test flow for a full page, a fragment, and redirects.

## Current Priorities

- Keep the architecture tight around server-side rendering and server-owned state.
- Start with Actix, but avoid coupling the core to Actix-specific concerns.
- Refine the `ssw-html` authoring experience around real page and layout examples before committing to deeper renderer internals.
- Keep improving `ssw-html` around app-shaped use cases, not isolated rendering helpers.
- Keep the Actix integration thin and ergonomic, with helpers that expose the HTML model cleanly instead of hiding it.
- Treat HTML semantics as first-class, not just string rendering ergonomics.

## Open Questions

- Whether `ssw-core::Render` remains the right cross-crate rendering boundary once a richer HTML DSL exists.
- How much form and validation support should live in core versus adapter crates.
- Whether `ssw-html` should continue evolving its own macro parser or eventually absorb code from a Maud-derived implementation.
- How `#id` shorthand and explicit `id=...` attributes should compose, if at all.
- What additional workflows the first Actix-backed example should cover next, such as forms or flash-style redirects.
- Whether boolean attr handling needs a more explicit opt-in for edge cases outside standard HTML boolean attributes.

## Next Likely Steps

- Expand `ssw-html` with more real-world ergonomics, such as id composition rules, reusable layout helpers, and clearer fragment helpers.
- Extend the Actix-backed example coverage toward form flows and mutation responses.
- Revisit the `ssw-core` rendering boundary once `ssw-html` and Actix usage put more pressure on it.
