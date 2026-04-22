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
- `ssw-html` now exposes a small `fonts` helper module for both Google Fonts links and local `@font-face` plus preload markup.
- `ssw-html` has now been exercised through an end-to-end Actix test flow for a full page, a fragment, and redirects.
- `ssw-html` and `ssw-actix` now have an end-to-end POST form flow with invalid redisplay and success redirect coverage.
- `ssw-html` and `ssw-actix` now have field-level validation coverage with accessible error markup and preserved input state.
- `ssw-components` now exposes a small reusable field helper layer, and the Actix-backed contact form now uses it instead of handwritten accessibility markup.
- `ssw-core` now models redirect-carried flash messages, and `ssw-actix` now reads and clears them through a cookie-backed request context.
- `ssw-actix` now exposes cookie-backed CSRF hooks through a `RequestContext`, and the contact form now uses hidden CSRF fields plus request-time verification.
- The repository now has a dedicated `COMPONENTS.md` design document that defines the `ssw-components` styling contract, token strategy, Base UI-inspired positioning, and initial component scope.
- The current `ssw-components` helpers now implement the styling contract with stable `ssw-*` classes, slot class names, and `data-invalid` or `data-level` hooks.
- `ssw-components` is now explicitly unstyled by default, and the first-party visual layer now lives separately at `styles/ssw-theme-default.css`.
- `ssw-components` now also includes the first layout and action primitives: `button`, `submit_button`, `container`, `section`, and `stack`.
- `ssw-components` now includes a native `select` helper with stable classes and selected-option handling driven by the current field value.
- A workspace example app now exists at `examples/ssw-intake-demo`, exercising the current component, flash, CSRF, layout, and stylesheet stack in a real binary.
- The example app now includes a dedicated `/style-guide` route so visual review of the current primitives does not depend on the intake flow alone.
- The repo now includes `scripts/capture-intake-demo.sh` so visual review can produce repeatable screenshots for `/` and `/style-guide`.
- The optional default theme and example shell have now had an initial screenshot-driven polish pass, improving card surfaces, control states, and page rhythm without growing the component API.
- The optional default theme has now had a second screenshot-driven pass toward a more restrained `shadcn/ui`-like direction, with denser spacing, lighter shadows, flatter surfaces, and quieter page chrome.
- A review and polish pass tightened the public API: plain strings flowing into `Markup` now escape by default, error-level notices now use `role="alert"`, and the README plus doc comments now better distinguish escaped text from trusted raw HTML.
- The public API across crates is now documented, and the README reflects the currently implemented slice instead of only the original architecture intent.
- `ssw-components` is now internally split into domain modules (`button`, `field`, `layout`, `notice`, `page`) while keeping the public API flat through re-exports, which should make the next phase of component work easier to scale.
- `ssw-components` now also includes the first reusable app-shell primitives (`page_shell`, `page_header`, `page_actions`, `card_header`), and the intake demo now uses them instead of app-local hero and card-heading markup.
- `ssw-components` now also includes a simple `top_nav` and `empty_state`, and a second example app now exists at `examples/ssw-projects-demo` to pressure list/detail/edit flows against the current shell, nav, empty-state, flash, and form primitives.
- `ssw-components` now also includes `link_button`, `MetaItem`, and `meta_list`, and both example apps now use those shared page-level helpers instead of app-local link and metadata markup.
- The screenshot capture workflow now supports wider configurable viewports and full-page mode, and the script now normalizes the output path to avoid relative-path failures with `agent-browser`.
- The repo now has an `SSW_CSS.md` design note for a proposed `ssw-css` companion crate, scoped narrowly around deterministic component-local CSS with plain browser CSS output and no runtime style injection.
- An initial experimental `ssw-css` crate now exists with a `css!` macro, deterministic class-based scoping, plain CSS output, `styles.classes(...)`, raw CSS-like declaration values, raw `@media` queries, and proof points in both the intake demo style guide and repeated card or badge UI inside the projects demo.
- The repo now has a minimal `ssw-workers` adapter, plus an `SSW_WORKERS.md` design note that keeps the backend scoped narrowly around Cloudflare Workers request/response integration rather than broad backend abstraction.
- `ssw-workers` now converts `ssw-core::Response` into `worker::Response`, exposes a cookie-backed flash and CSRF `RequestContext`, and is proven through both wasm checks and a locally runnable `wrangler dev` flow in `examples/ssw-workers-demo`.
- The Worker demo now serves the first-party theme CSS from a Worker route, which gives the backend a very small but real asset path without introducing a general asset pipeline yet.
- `ssw-core` now owns a backend-neutral `RequestState` for flash and CSRF request state, while Actix and Workers keep only cookie parsing, token generation, and response cookie application.
- `ssw-core` now also owns status-bearing HTML and text responses with 200 defaults, and both Actix and Workers now map those statuses through their native response types.
- The Worker demo now exercises both a fragment endpoint and a shared-model HTML 404 page, which confirms that fragments and non-200 HTML now fit the current shared response model cleanly.

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
- Consolidate repeated SSR form markup in `ssw-components` before considering heavier form abstractions in `ssw-core`.
- Keep current flash and CSRF support positioned as hooks, not a complete session or secret-management layer yet.
- Treat styling conventions, slot names, and state attributes in `ssw-components` as a real API surface before expanding the component catalog.
- Keep any future `ssw-css` work narrow enough that plain CSS remains a first-class path and the primitive component layer does not depend on scoped-style tooling.
- Keep `ssw-workers` adapter-first, so Cloudflare runtime details do not leak into `ssw-core` while the backend remains a narrow proof.

## Open Questions

- Whether `ssw-core::Render` remains the right cross-crate rendering boundary once a richer HTML DSL exists.
- How much form and validation support should live in core versus adapter crates.
- Whether `ssw-html` should continue evolving its own macro parser or eventually absorb code from a Maud-derived implementation.
- How `#id` shorthand and explicit `id=...` attributes should compose, if at all.
- Whether remote and local font helpers should eventually sit behind a broader asset pipeline with build-time fetching and self-hosting.
- Whether flash transport should stay cookie-backed and unsigned, or move behind a more explicit application secret/session abstraction.
- Whether the current shared `RequestState` boundary is enough, or whether any more of the request-context model should move into `ssw-core` without forcing a lowest-common-denominator backend abstraction.
- What the next mutation-oriented step should be after flash and CSRF hooks, such as a larger form abstraction or richer request context primitives.
- Whether the first default theme should live as a separate crate, a plain CSS package, or example-app assets first.
- Which parts of the current Actix-shaped flash, CSRF, cookie, and request-context model survive the initial Cloudflare Workers adapter cleanly, and which ones still leak assumptions.
- How far `ssw-css` should grow beyond the current prototype, especially around selector coverage, scoping keys, whether the current `1 rem` dimension syntax is acceptable, and whether extraction can arrive without making debugging worse.
- Whether boolean attr handling needs a more explicit opt-in for edge cases outside standard HTML boolean attributes.
- Which currently explicit example patterns should become first-class helpers without bloating the public API.

## Next Likely Steps

- Expand `ssw-html` with more real-world ergonomics, such as id composition rules, reusable layout helpers, and clearer fragment helpers.
- Revisit the mutation layer now that flash messages and CSRF hooks exist, especially around whether any of that API should move into `ssw-core`.
- Pressure the new example app and `/style-guide` route until they reveal what should change in component APIs, request context, and asset ergonomics.
- Keep the visual feedback loop cheap: live preview, style-guide route, and scripted screenshots should stay working as the primary refinement workflow.
- Keep the primitive layer structurally stable while iterating on the optional default theme separately.
- Decide whether the current `ssw-css` prototype is good enough to keep expanding, or whether the API should stay frozen until more example-app pressure justifies broader CSS support.
- Use the locally runnable `ssw-workers` proof to decide what, if anything, should move out of adapter crates before the backend grows further.
- Continue tightening the shared backend boundary only where both Actix and Workers already prove the same state or semantics, rather than abstracting ahead of real pressure.
- Add the next plain-HTML primitives that still fit the no-JS baseline cleanly, such as metadata-heavy detail helpers or link-style action variants that prove themselves through the example apps.
- Revisit the `ssw-core` rendering boundary once `ssw-html` and Actix usage put more pressure on it.
