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

- decision: Allow `.class` shorthand and explicit `class=(...)` attributes to compose into a single class list, rather than treating them as conflicting syntax.
  evidence: [`crates/ssw-html/src/lib.rs`](../crates/ssw-html/src/lib.rs)
  evidence: [`crates/ssw-html-macros/src/lib.rs`](../crates/ssw-html-macros/src/lib.rs)

- decision: Keep `ssw-actix` thin, with convenience helpers like `page`, `fragment`, and `redirect`, plus `Responder` support for `ActixResponse`, rather than inventing a larger adapter layer before real pressure appears.
  evidence: [`crates/ssw-actix/src/lib.rs`](../crates/ssw-actix/src/lib.rs)

- decision: Expression-based boolean HTML attributes in `ssw-html` should follow HTML presence semantics for standard boolean attributes, while non-boolean attributes such as `aria-*` keep explicit `"true"` and `"false"` string values.
  evidence: [`crates/ssw-html/src/lib.rs`](../crates/ssw-html/src/lib.rs)

- decision: Use Actix-backed end-to-end form flows to shape the next abstraction layer, and avoid inventing generic form APIs in `ssw-core` until invalid redisplay, field errors, and redirect flows have more real usage pressure.
  evidence: [`crates/ssw-actix/src/lib.rs`](../crates/ssw-actix/src/lib.rs)

- decision: Keep field-level validation logic explicit in the Actix-backed example for now, including per-field errors, `aria-invalid`, and `aria-describedby`, so the next abstraction layer is based on real SSR form markup patterns rather than guessed APIs.
  evidence: [`crates/ssw-actix/src/lib.rs`](../crates/ssw-actix/src/lib.rs)

- decision: Keep macro-expansion support types in `ssw-html` public but documented as implementation details, so the user-facing API stays focused while the proc-macro still expands cleanly in downstream crates.
  evidence: [`crates/ssw-html/src/lib.rs`](../crates/ssw-html/src/lib.rs)

- decision: Keep `ssw-components` aligned with the public `ssw-html` API by using `html!` and the document builder internally instead of hand-built raw HTML strings.
  evidence: [`crates/ssw-components/src/lib.rs`](../crates/ssw-components/src/lib.rs)

- decision: Extract the first reusable SSR form-field layer in `ssw-components`, not `ssw-core`, so the API stays driven by real Actix-backed form markup before any broader form abstraction is committed.
  evidence: [`crates/ssw-components/src/lib.rs`](../crates/ssw-components/src/lib.rs)
  evidence: [`crates/ssw-actix/src/lib.rs`](../crates/ssw-actix/src/lib.rs)
