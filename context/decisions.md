# Decisions

## 2026-04-09

- decision: Start with an Actix-first implementation, but keep the architecture open to additional backends later through narrow crate boundaries rather than premature universal abstractions.
  evidence: [`ARCHITECTURE.md`](../ARCHITECTURE.md)

- decision: Use four initial crates, `ssw-core`, `ssw-actix`, `ssw-html`, and `ssw-components`.
  evidence: [`ARCHITECTURE.md`](../ARCHITECTURE.md)

- decision: Keep the component collection as an optional layer, separate from the framework core, even if it lives in the same repository.
  evidence: [`ARCHITECTURE.md`](../ARCHITECTURE.md)

- decision: Treat Maud as the starting point for the HTML layer, but do not commit to a full fork until the required semantic changes are identified.
  evidence: [`ARCHITECTURE.md`](../ARCHITECTURE.md)

- decision: For the first compileable slice, keep `ssw-html` as a tiny owned markup and escaping layer instead of importing or forking Maud immediately.
  evidence: [`crates/ssw-html/src/lib.rs`](../crates/ssw-html/src/lib.rs)

- decision: Use `ssw_core::Render` as the initial cross-crate rendering boundary, with `ssw-actix` converting `ssw-core::Response` values into `actix_web::HttpResponse`.
  evidence: [`crates/ssw-core/src/lib.rs`](../crates/ssw-core/src/lib.rs)
  evidence: [`crates/ssw-actix/src/lib.rs`](../crates/ssw-actix/src/lib.rs)

- decision: Shape `ssw-html` around a Maud-like syntax and authoring experience, but do not preserve Maud compatibility as a goal.
  evidence: user direction on 2026-04-09

- decision: Keep the public HTML API in `ssw-html`, with an internal `ssw-html-macros` proc-macro crate used as an implementation detail.
  evidence: [`crates/ssw-html/src/lib.rs`](../crates/ssw-html/src/lib.rs)
  evidence: [`crates/ssw-html-macros/src/lib.rs`](../crates/ssw-html-macros/src/lib.rs)

- decision: Add page-oriented ergonomics directly in `ssw-html`, including a small `Document` builder and macro support for empty tags plus shorthand selectors, to keep the API grounded in real SSR page authoring.
  evidence: [`crates/ssw-html/src/lib.rs`](../crates/ssw-html/src/lib.rs)
  evidence: [`crates/ssw-html-macros/src/lib.rs`](../crates/ssw-html-macros/src/lib.rs)

- decision: Support optional attribute omission through `Option<T>` attribute expressions instead of forcing conditional markup around every optional attribute.
  evidence: [`crates/ssw-html/src/lib.rs`](../crates/ssw-html/src/lib.rs)
