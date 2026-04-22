# ssw-workers Design

## Current status

`ssw-workers` now exists as a minimal backend proof in the workspace.

The current implementation covers:

- conversion from `ssw-core::Response` into `worker::Response`
- page, fragment, and redirect helpers
- cookie-backed flash and CSRF request context
- a minimal Worker example with GET, POST, redirect, flash flow, and a Worker-served stylesheet route
- a local `wrangler dev` workflow that has been exercised against the example

What it does not cover yet:

- static asset serving
- deployed Worker verification
- Cloudflare product bindings such as D1, KV, R2, or Durable Objects
- a generalized backend abstraction beyond the current response and request seams

## Purpose

`ssw-workers` should be a narrow Cloudflare Workers adapter for `ssw-rs`.

The goal is simple:

- render `ssw-rs` HTML documents and fragments from a Worker
- preserve the framework's HTML-first, server-owned model
- keep Cloudflare-specific runtime details out of `ssw-core`

This should not be treated as a general backend abstraction exercise. It should be treated as a focused deployment target for edge-rendered `ssw-rs` apps.

## Why this backend is worth considering

Cloudflare officially supports Rust Workers through `workers-rs`, and the current docs position Rust as a first-class Workers language with bindings for platform products such as KV, R2, Queues, D1, and service bindings.

Relevant sources:

- Cloudflare Workers Rust docs:
  https://developers.cloudflare.com/workers/languages/rust/
- `workers-rs` repository:
  https://github.com/cloudflare/workers-rs
- `worker` crate docs:
  https://docs.rs/worker/latest/worker/

This is a strategically strong fit for `ssw-rs` because:

- `ssw-rs` is HTML-first, not hydration-first
- server-owned logic maps well to request/response Workers
- edge deployment is useful for durable SSR apps that do not require a long-lived process

## Constraints that matter

Cloudflare Workers is not just another HTTP server target.

Important runtime constraints from the current docs and `workers-rs` README:

- Worker code must compile to `wasm32-unknown-unknown`
- normal threaded runtimes such as Tokio and `async_std` are not supported
- bundle size matters much more than on a conventional server
- some APIs are still missing or rough, and `workers-rs` explicitly calls out rough edges and unimplemented APIs

Relevant source passages:

- Cloudflare docs: Rust support is through `workers-rs`, with `Request`, `Env`, `Context`, and `Response` in the fetch handler model
  https://developers.cloudflare.com/workers/languages/rust/
- `workers-rs` README and FAQ:
  https://github.com/cloudflare/workers-rs

Bluntly, this means `ssw-workers` should not try to simulate Actix.

## Naming

The adapter should be explicit:

- `ssw-workers`

Do not name it something generic like `ssw-edge`.

Reason:

- this adapter is specific to the `worker` crate and Cloudflare runtime model
- explicit naming keeps the backend boundary honest

## Responsibilities

### `ssw-workers`

Should own:

- conversion from `ssw-core` responses into `worker::Response`
- request-side helpers for reading method, path, query, headers, and form data
- cookie and header helpers where Workers-specific behavior is required
- minimal integration helpers for rendering pages, fragments, redirects, and errors

Should not own:

- HTML rendering logic
- app-level flash/session design in core
- Cloudflare product clients inside `ssw-core`
- a generic abstraction for all edge runtimes

## What should stay out of `ssw-core`

The following should remain adapter-local or app-local:

- `Env` bindings for KV, D1, R2, Queues, Durable Objects, AI, or Hyperdrive
- `Context::waitUntil` integration
- service-binding specifics
- any `worker::Router`-specific ergonomics

These are useful runtime features, but they are not part of the stable cross-backend core.

## Recommended scope for v0

Start with the smallest slice that proves the backend is real.

### v0 should support:

1. full document responses
2. HTML fragment responses
3. redirects
4. status codes and headers
5. cookie read/write helpers
6. a minimal GET and POST roundtrip example

### v0 should not try to support:

- D1 helpers in core
- KV-backed sessions
- Durable Object abstractions
- `waitUntil` integration in core
- streaming abstractions unless the basic response path demands them
- a complete replacement for the current Actix request context

## Relationship to the current `ssw-core`

The current crate split is a good starting point, but it is not yet enough proof that the backend boundary is correct.

The Workers adapter should pressure these specific seams:

- response conversion
- document vs fragment distinction
- redirect semantics
- header and cookie manipulation
- form-body extraction

Only after those are proven in both Actix and Workers should anything move or generalize in `ssw-core`.

## Flash and CSRF implications

This is where the second backend is actually useful.

Current `ssw-rs` flash and CSRF support is only proven through the Actix request context. A Workers adapter will force a more honest boundary.

The likely outcome is:

- keep flash and CSRF concepts in `ssw-core` or a shared layer only if they are truly backend-neutral
- keep cookie transport and request extraction logic adapter-specific

That is a good pressure test. It is more valuable than adding another component right now.

## Example architecture for a Worker app

The rough shape should stay close to the Workers model:

```rust
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    ssw_workers::respond(req, env, ctx, |request| async move {
        // app routing and ssw-rs rendering here
    }).await
}
```

The adapter can offer helpers, but the underlying Workers fetch model should stay visible.

That is the right tradeoff:

- ergonomic enough to use
- explicit enough to debug

## Routing strategy

Do not try to invent a router in `ssw-workers`.

Options:

- use `worker::Router` directly when it is enough
- or keep routing app-local until a repeated pattern is obvious

Reason:

- routing is one of the easiest ways to accidentally over-abstract
- the current objective is response and request integration, not framework-within-a-framework layering

## Asset and CSS story on Workers

The Workers adapter should not become an asset pipeline.

It only needs a minimal story:

- serving or referencing static assets in a Worker-compatible way
- ensuring `ssw-html` and `ssw-css` output can be linked or embedded cleanly

The current demo proves a very small asset path by serving the first-party theme CSS from a Worker route. That is useful, but it is not a full asset pipeline yet.

## Performance and size concerns

Workers puts more pressure on:

- Wasm binary size
- dependency weight
- startup cost

This matters for `ssw-rs` because:

- a second backend should not drag in unnecessary server-oriented dependencies
- core abstractions should stay lean enough to compile well to Wasm

This is another reason to keep `ssw-workers` narrowly scoped and to resist moving runtime-specific helpers into core prematurely.

## Recommended v0 milestone

The first milestone should be intentionally small.

This is now implemented:

1. `ssw-workers` exists
2. `ssw-core::Response` converts into `worker::Response`
3. one Worker example proves:
   - page render
   - form POST
   - redirect
   - flash notice
   - CSRF verification

The next step is not more surface area by default. It is to note exactly which current Actix assumptions still leak and only then decide whether any shared boundary should move.

## Open questions

- Should `ssw-workers` expose a small `respond(...)` helper, or should it stay as manual conversions around `worker::Response` for longer?
- Which parts of the current flash and CSRF model are truly backend-neutral?
- Does Workers need a different cookie abstraction than Actix for the shared response model to stay simple?
- Should the first Worker example use only plain request/response flows, or also prove one binding such as D1?
- At what point is it worth introducing a static asset helper for Worker-hosted apps?
