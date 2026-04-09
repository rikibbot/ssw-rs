# Current Context

## Status

- The repository is initialized with a project README and an architecture document.
- A repo-specific `AGENTS.md` and lightweight `context/` system are in place.
- No Cargo workspace or crates exist yet.
- The agreed initial crate layout is `ssw-core`, `ssw-actix`, `ssw-html`, and `ssw-components`.

## Current Priorities

- Keep the architecture tight around server-side rendering and server-owned state.
- Start with Actix, but avoid coupling the core to Actix-specific concerns.
- Treat the HTML layer as Maud-derived or Maud-inspired until the exact fork needs are clear.

## Open Questions

- Whether `ssw-html` should begin as a wrapper around Maud or a minimal maintained fork.
- What the first stable response and rendering traits in `ssw-core` should look like.
- How much form and validation support should live in core versus adapter crates.

## Next Likely Steps

- Scaffold the Cargo workspace and empty crates.
- Define initial crate dependency boundaries.
- Prototype the first end-to-end rendered Actix page.
