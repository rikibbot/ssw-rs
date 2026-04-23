# ssw-css Design

## Purpose

`ssw-css` should be a small companion styling layer for `ssw-rs`.

Current state: an initial experimental prototype now exists in `crates/ssw-css`. The implementation is intentionally narrower than the full design note:

- string-literal selectors
- raw CSS-like declaration values
- local class selector rewriting
- raw `@media` queries
- `styles.class(...)`, `styles.classes(...)`, `render()`, and `style_tag()`

It is currently proven in two places:

- a small isolated block in the intake style guide
- app-owned project-card and status-badge UI in the projects demo, via `examples/ssw-projects-demo/src/components.rs`

The goal is not to replace CSS. The goal is to make component-local styling more ergonomic while still emitting plain, predictable CSS for the browser.

Short version:

- author styles close to the Rust component
- keep output as normal CSS
- keep scoping deterministic
- avoid runtime style injection and client-side magic

## Problem statement

`ssw-rs` currently has a good story for:

- HTML authoring in Rust
- server-rendered components
- stable component anatomy
- linked stylesheets and optional themes

What it does not yet have is a good story for styles that belong to a single component or page fragment.

Plain CSS files still work, but they have real friction:

- class naming drifts over time
- local styles and markup drift apart
- styling a small component often requires editing a separate global stylesheet
- reuse across crates is awkward without stronger scoping

That is the gap `ssw-css` should address.

## Goals

- Provide scoped component CSS with deterministic class names.
- Keep the browser-facing output as plain CSS.
- Compose cleanly with `ssw-html` and `ssw-components`.
- Work with server-rendered HTML and no client runtime.
- Keep the API small and obvious.
- Leave room for static extraction later.

## Non-goals

- CSS-in-JS runtime injection
- client-side style reactivity
- replacing normal CSS files for global app styles
- a utility-first framework
- a full design token engine
- a custom cascade model that fights the web platform

## Core principle

`ssw-css` should make local CSS easier, not invent a new styling system.

If an app can already express something clearly with plain CSS, `ssw-css` should not try to outsmart it.

## Recommended scope for v0

Start with scoped component CSS only.

That means:

- scoped class generation
- nested selectors relative to the component root
- pseudo-classes and pseudo-elements
- media queries
- CSS custom properties

Do not start with:

- global resets
- theme tokens
- keyframe management beyond plain passthrough
- automatic critical CSS
- cross-component selector magic

## Proposed crate boundary

Add a separate crate:

- `ssw-css`

Responsibilities:

- authoring scoped CSS in Rust
- generating stable scoped class names
- rendering CSS strings or style blocks
- later, optionally supporting static extraction

It should not depend on Actix.

It should likely remain independent from `ssw-components`, because components should stay usable with plain CSS and without any special styling runtime.

## Proposed authoring model

The right shape is a small `css!` macro plus a tiny runtime value.

Example:

```rust
use ssw_css::css;
use ssw_html::html;

let styles = css! {
    ".root" {
        display: grid;
        gap: 1 rem;
    }

    ".title" {
        font-weight: 600;
        letter-spacing: -0.02 em;
    }

    ".root:hover .title" {
        color: var(--accent);
    }
};

html! {
    section class=(styles.class("root")) {
        h2 class=(styles.class("title")) { "Hello" }
    }
}
```

Possible rendered class names:

- `root` -> `sswc-abc123-root`
- `title` -> `sswc-abc123-title`

The exact prefix matters less than the properties:

- stable for identical source
- deterministic across requests
- readable enough for debugging

Current caveat:

- CSS dimensions must currently be written as `1 rem`, `0.75 rem`, or `-0.02 em` because Rust tokenization does not allow tokens like `1rem` directly inside a macro input. `ssw-css` joins them back into normal CSS output.

## Public API shape

The initial API should stay narrow:

- `css! { ... }` to define a stylesheet
- `styles.class("slot")` to resolve a scoped class
- `styles.classes([...])` to join multiple scoped classes
- `styles.render()` to emit CSS text
- `styles.style_tag()` to emit a `<style>` block through `ssw-html`

Possible future additions:

- `styles.id("slot")` only if there is a clear need
- extraction-oriented APIs for build tooling

Avoid builder-heavy APIs in v0. The macro should carry most of the ergonomics.

## Scoping model

Scoping should be explicit and class-based.

Rules:

1. Authors refer to local selectors like `.root`, `.title`, `.error`.
2. `ssw-css` rewrites those to deterministic scoped classes.
3. Global selectors are disallowed or very tightly controlled in v0.
4. Descendant selectors should remain plain CSS after rewriting.

Examples:

- `.root` -> `.sswc-abc123-root`
- `.root .title` -> `.sswc-abc123-root .sswc-abc123-title`
- `.root[data-open="true"]` remains valid after rewriting

This keeps the model simple and inspectable.

## Output model

`ssw-css` should support two output paths over time, but only one is required at first.

### v0

Render inline style blocks or CSS strings:

- `styles.render()` for raw CSS
- `styles.style_tag()` for direct use in `ssw-html`

This is enough to validate ergonomics.

### Later

Support static extraction:

- collect styles at build time
- emit a CSS asset
- keep the same class-name resolution API

That later step matters because large apps should not rely on scattered inline `<style>` blocks forever.

## Relationship to `ssw-html`

`ssw-html` should remain responsible for:

- HTML authoring
- documents
- head helpers
- asset and style-tag inclusion

`ssw-css` should remain responsible for:

- scoped CSS authoring
- selector rewriting
- CSS rendering

This keeps responsibilities clean.

`ssw-html` may eventually expose convenience integration, but it should not absorb scoped CSS authoring directly unless the split proves unnecessary.

## Relationship to `ssw-components`

`ssw-components` should not depend on `ssw-css` for correctness.

That boundary matters.

The component layer should continue to expose:

- semantic HTML
- stable slot classes
- predictable DOM structure
- `data-*` state hooks

`ssw-css` should be an optional authoring tool for app-specific components or future higher-level component packages, not a requirement for the primitive layer.

## Relationship to the default theme

The first-party theme should still exist as plain CSS.

Reason:

- it should be easy to link, override, and inspect
- it should not require Rust-side style generation
- it should remain usable by downstream apps that do not want local scoped CSS

So the split should remain:

- `ssw-components`: unstyled structure
- `ssw-theme-default`: plain optional CSS theme
- `ssw-css`: optional scoped authoring tool

## Tradeoffs

### Why this is worth doing

- Local component styling becomes much easier to reason about.
- The authoring model becomes more symmetrical with `ssw-html`.
- Class collisions become much less likely.
- Component examples can carry their own styles without growing global CSS files too quickly.

### Why this is risky

- It can turn into a second language if the scope is not kept tight.
- Poor scoping rules would make debugging harder, not easier.
- If static extraction never arrives, large apps may end up with too many inline styles.
- If the macro becomes too broad, maintenance cost will spike.

## Decision guardrails

`ssw-css` should only move forward if it stays true to these rules:

1. No client runtime is required.
2. The browser still receives normal CSS.
3. Generated selectors are deterministic and debuggable.
4. Plain CSS remains a first-class path.
5. The primitive component layer does not depend on it.

If those rules stop being true, the project should fall back to plain CSS files instead of carrying a more complex system.

## Suggested v0 implementation plan

1. Create a tiny `ssw-css` crate.
2. Support `css!` with local class selectors only.
3. Add `styles.class(...)`, `styles.classes(...)`, `render()`, and `style_tag()`.
4. Prove it in a small example component, not in `ssw-components` first.
5. Validate whether inline rendering is ergonomic enough before designing extraction.

## Open questions

- Should scoped styles be keyed from source text, call site, explicit module name, or a mix of those?
- Should v0 allow any global selectors such as `:root` or `@font-face`?
- Should `style_tag()` live in `ssw-css` or be an integration helper in `ssw-html`?
- Is nested syntax worth supporting immediately, or should v0 stay close to flat CSS rules?
- When static extraction arrives, should it be a library feature, a build script, or a separate tool?
