# ssw-rs Architecture

## Product thesis

`ssw-rs` is an HTML-first Rust web framework for building long-lived web apps with server-side rendering as the default model.

The framework should optimize for:

- predictable request and response flow
- simple mental models
- durable codebases that do not depend on heavy client runtimes
- progressive enhancement when JavaScript improves the experience
- strong ergonomics for composing HTML in Rust

The framework should not assume that every app wants client-side routing, hydration-heavy components, or duplicated server and client state.

## Goals

- Render full HTML documents on the server by default
- Support partial rendering as an explicit feature, not the main model
- Keep the core independent from any single HTTP framework
- Start with a first-party Actix integration
- Make forms, redirects, validation, and flash-style UX first-class
- Allow optional client enhancements without breaking no-JS behavior
- Keep the dependency surface small and understandable

## Non-goals for v0

- Client-side routing
- SPA-style state synchronization
- Mandatory hydration
- Realtime primitives beyond what an adapter can already expose
- A backend-agnostic abstraction so wide that it weakens Actix ergonomics

## Design principles

### 1. HTML-first

The primary output is HTML. Pages should be rendered in a form that is immediately useful to the browser without additional client bootstrapping.

### 2. Server-owned state

Application state and control flow live on the server. Client JavaScript may enhance interaction, but should not become the source of truth for core behavior.

### 3. Explicit enhancement

JavaScript should be additive and local. A component must render meaningful HTML before any enhancement layer runs.

### 4. Minimal core

The core should define rendering and response concepts, not a complete reinvention of an HTTP stack.

### 5. Strong boundaries

Adapters, HTML rendering, and components should live in separate crates so they can evolve independently.

## Crate layout

### `ssw-core`

Responsibilities:

- shared response and rendering traits
- document and fragment abstractions
- shared error types
- redirect and flash-style response helpers
- form and validation primitives that do not depend on Actix
- utilities for content type and response metadata

This crate should avoid depending on Actix or any HTML DSL implementation.

### `ssw-actix`

Responsibilities:

- Actix request and response integration
- conversions from `ssw-core` response types into Actix responders
- route-level helpers for rendering pages and fragments
- extraction hooks where Actix-specific behavior is needed

This crate is where framework-specific ergonomics live. It should feel native to Actix rather than pretending every backend has identical capabilities.

### `ssw-workers`

Responsibilities:

- Cloudflare Workers request and response integration
- conversion from `ssw-core` response types into `worker::Response`
- request helpers where Workers-specific extraction is needed
- minimal rendering and redirect ergonomics that feel native to the Workers fetch model

This crate should remain narrow. It should not try to hide the Cloudflare runtime model, and it should not pull Cloudflare product bindings into `ssw-core`.

Current status:

- a minimal adapter exists
- `ssw-core::Response` now converts into `worker::Response`
- cookie-backed flash and CSRF request context is proven on Workers
- a small GET or POST example exists, but broader deployment ergonomics and asset handling are still intentionally out of scope

### `ssw-html`

Responsibilities:

- Rust HTML DSL
- escaping and rendering implementation
- typed helpers for elements, attributes, fragments, and layouts
- framework-oriented extensions such as document shells, assets, and script/style helpers

This layer should likely begin as a Maud-derived or Maud-inspired implementation. The goal is to preserve simplicity while giving `ssw-rs` control over the rendering model and future ergonomics.

Initial rule: fork only the minimum necessary. If Maud's current semantics remain sufficient, keep the divergence small and intentional.

### `ssw-components`

Responsibilities:

- carefully designed, production-ready UI components
- form controls, navigation, feedback, layouts, and common application shells
- optional enhancement hooks for richer behavior

This crate must remain an optional layer. It should prove the framework's UX direction, not define the framework's core architecture.

### `ssw-css`

Responsibilities:

- scoped CSS authoring for component-local styles
- deterministic scoped class generation
- rendering plain CSS strings or style blocks
- later, optional static extraction

This crate should remain optional. It should improve component-local CSS ergonomics without introducing a client runtime or replacing plain linked stylesheets as a first-class path.

Current state: an initial experimental crate exists with deterministic class-based scoping, plain CSS output, `@media` support, and inline style-tag rendering. Static extraction and broader CSS coverage are still future work.

## Backend strategy

`ssw-rs` should start with Actix because:

- it is mature and proven
- it provides good performance and routing primitives
- it is a practical place to validate the framework shape

To keep the door open for additional backends later, the core should define a narrow set of backend-facing interfaces:

- renderable page or fragment outputs
- redirect and error response semantics
- request-scoped context hooks where absolutely necessary

The system should not attempt a universal adapter model in v0. Add a second backend only after the Actix integration proves which abstractions are genuinely stable.

## Rendering model

The rendering system should distinguish clearly between:

- full documents
- partial fragments
- plain text or non-HTML responses

That distinction matters because pages and fragments often have different layout, asset, and caching needs.

Desired properties:

- zero-surprise escaping
- cheap composition of reusable fragments
- layout composition without macro abuse
- simple generation of `head` assets and per-page scripts

## Forms and mutations

Forms are central to the framework's value.

The framework should provide first-class support for:

- parsing form submissions
- validation with structured field errors
- redisplaying invalid forms with preserved input
- redirects after successful mutations
- flash-style success and error messages
- CSRF hooks

This is a better differentiator than trying to compete on client interactivity alone.

## Progressive enhancement

Enhancement should be possible without forcing a JavaScript-heavy architecture.

Rules:

- every enhanced component must have a useful no-JS baseline
- enhancement APIs should be opt-in
- enhancement code should attach to server-rendered HTML, not replace it
- component behavior should degrade cleanly

`Lit` may be a useful fit for some components, but it must remain optional and subordinate to the SSR model.

## Maud strategy

Maud is a strong starting point because it is small, understandable, and already aligned with HTML generation in Rust.

Recommended approach:

1. Begin with a close read of Maud's rendering, escaping, and macro architecture.
2. Identify the exact changes needed for `ssw-rs`.
3. Fork only if those changes require owning the implementation.
4. Keep the fork surface documented so divergence remains deliberate.

Reasons to own a fork might include:

- component ergonomics that Maud should not absorb
- document and fragment semantics tailored to `ssw-rs`
- asset and script helpers integrated into the DSL
- long-term control over dependency surface and release cadence

If those reasons do not materialize, a thin wrapper may be the better engineering choice.

## v0 feature set

The first milestone should be intentionally small:

1. `ssw-html` can render documents and fragments
2. `ssw-actix` can return rendered HTML from handlers
3. layout and partial composition work cleanly
4. forms support validation and error redisplay
5. redirects and flash messages are ergonomic
6. `ssw-components` ships a small polished starter set

Suggested starter components:

- button
- input
- textarea
- select
- form field wrapper with label and error state
- alert or notice
- top navigation
- page container and section layout

## Repository roadmap

### Phase 1

- create workspace and crate skeletons
- define `ssw-core` response traits and types
- prototype `ssw-html` from Maud or a minimal fork
- build a single end-to-end page in Actix

### Phase 2

- implement forms, validation, redirects, and flash messages
- add fragment rendering patterns
- create an example app that exercises real workflows

### Phase 3

- stabilize the public API
- expand `ssw-components`
- evaluate whether a second backend is justified

## Open questions

- How much of the HTML DSL should rely on macros versus plain Rust types and builders?
- Should assets be handled directly in `ssw-html` or in a higher application layer?
- What is the minimum adapter API that still feels native in Actix?
- Which form validation story best balances ergonomics and dependency weight?
- Where should partial page update patterns live, core or adapter?
