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
- `ssw-html` and `ssw-actix` now have an end-to-end POST form flow with invalid redisplay and success redirect coverage.
- `ssw-html` and `ssw-actix` now have field-level validation coverage with accessible error markup and preserved input state.
- The public API across crates is now documented, and the README reflects the currently implemented slice instead of only the original architecture intent.

## Current Priorities

- Keep the architecture tight around server-side rendering and server-owned state.
- Start with Actix, but avoid coupling the core to Actix-specific concerns.
- Refine the `ssw-html` authoring experience around real page and layout examples before committing to deeper renderer internals.
- Keep improving `ssw-html` around app-shaped use cases, not isolated rendering helpers.
- Keep the Actix integration thin and ergonomic, with helpers that expose the HTML model cleanly instead of hiding it.
- Treat HTML semantics as first-class, not just string rendering ergonomics.
- Pressure the stack with realistic mutation flows before designing larger abstractions for forms, validation, or flash state.
- Prefer app-shaped validation patterns that expose what abstractions are actually missing, rather than inventing them upfront.
- Keep the documented public API small and intentional, especially around `ssw-html` macro internals.

## Open Questions

- Whether `ssw-core::Render` remains the right cross-crate rendering boundary once a richer HTML DSL exists.
- How much form and validation support should live in core versus adapter crates.
- Whether `ssw-html` should continue evolving its own macro parser or eventually absorb code from a Maud-derived implementation.
- How `#id` shorthand and explicit `id=...` attributes should compose, if at all.
- What the next mutation-oriented step should be after field-level validation, such as flash messages, CSRF hooks, or reusable field helpers.
- Whether boolean attr handling needs a more explicit opt-in for edge cases outside standard HTML boolean attributes.
- Which currently explicit example patterns should become first-class helpers without bloating the public API.

## Next Likely Steps

- Expand `ssw-html` with more real-world ergonomics, such as id composition rules, reusable layout helpers, and clearer fragment helpers.
- Extend the Actix-backed example coverage toward flash messages, CSRF hooks, and reusable mutation UX patterns.
- Revisit the `ssw-core` rendering boundary once `ssw-html` and Actix usage put more pressure on it.
