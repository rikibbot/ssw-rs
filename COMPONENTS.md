# ssw-components Design

## Purpose

`ssw-components` should be the durable UI foundation for `ssw-rs`.

It should aim for the same long-term qualities that make systems like Base UI compelling:

- careful component anatomy
- accessibility-first markup
- styling freedom
- stable composition boundaries
- no unnecessary coupling to a specific visual theme

The important difference is that `ssw-components` is being built for an HTML-first, Rust-first, server-rendered framework. That means the component model must start from durable server-rendered HTML, not from client-side hooks or hydration-driven state.

## Design goals

- Components render meaningful HTML before any JavaScript runs.
- Accessibility semantics are part of the component contract, not an optional add-on.
- Styling is explicit, stable, and easy to override.
- Components are useful with no default CSS, but there is a clear path for a first-party default theme.
- Advanced interaction stays optional and progressive.
- Component APIs stay small and predictable.

## Non-goals

- Rebuilding a React-style primitive system in Rust.
- Hiding HTML structure behind opaque component abstractions.
- Depending on client-side state for baseline functionality.
- Shipping a large visual design system before the styling contract is stable.
- Building complex interactive primitives before the enhancement model is proven.

## Relationship to Base UI

Base UI is a good reference for philosophy, not for direct implementation.

What to borrow:

- durable, accessibility-first component thinking
- slot-oriented anatomy
- separation of structure from theme
- explicit state hooks

What not to copy directly:

- React hooks and client-side state machines
- hydration-first interaction patterns
- primitives whose baseline behavior depends on JavaScript

If code is borrowed from external libraries in the future, it should be limited to narrowly scoped enhancement logic where the match is real and the licensing and maintenance burden are clear.

## Styling contract

Before `ssw-components` grows much further, the project should treat styling as a real API surface.

Each component should expose:

- stable root and slot class names
- stable semantic `data-*` state attributes
- accessible native HTML whenever possible
- predictable DOM structure

Recommended conventions:

- Root class: `ssw-{component}`
- Slot class: `ssw-{component}__{slot}`
- Variant/state class only when there is no better semantic hook
- Prefer `data-*` attributes for dynamic state

Examples:

- `class="ssw-field"`
- `class="ssw-field__label"`
- `class="ssw-field__error"`
- `data-invalid="true"`
- `data-variant="primary"`
- `data-size="sm"`

This gives downstream apps a stable styling surface without forcing a single CSS methodology.

## Token strategy

The first-party styling direction should be based on CSS custom properties, not CSS-in-JS.

Recommended token layers:

1. Global foundation tokens
   - color
   - typography
   - spacing
   - radius
   - shadow
   - duration
2. Semantic tokens
   - surface
   - text
   - border
   - accent
   - success
   - warning
   - danger
3. Component tokens
   - input height
   - button padding
   - field gap

Example names:

- `--ssw-color-text`
- `--ssw-color-surface`
- `--ssw-color-accent`
- `--ssw-space-2`
- `--ssw-radius-md`
- `--ssw-field-gap`

## CSS ownership

The framework should separate component structure from default visual styling.

Recommended direction:

- `ssw-components`: semantic HTML, accessibility semantics, stable classes and data attributes
- future `ssw-theme-default` or `ssw-styles-default`: optional first-party CSS package

This keeps the component crate durable and avoids locking the framework to a single visual taste too early.

Short version:

- structure lives in Rust
- theme lives in CSS

## DOM and state rules

Component output should follow a few strict rules:

1. Use native elements whenever possible.
2. Preserve expected browser behavior by default.
3. Prefer explicit attributes over implicit magic.
4. Keep DOM shallow unless a wrapper is needed for semantics or styling.
5. Encode interactive state in HTML attributes that CSS and enhancement code can both read.

Examples:

- invalid fields expose `aria-invalid="true"` and `data-invalid="true"`
- dismissible notices expose `data-dismissible="true"`
- enhanced menus may expose `data-open="true"`

## Enhancement model

`ssw-components` should treat JavaScript enhancement as a separate layer.

Rules:

- Every component must have a useful no-JS baseline.
- JavaScript may enhance, never replace, the server-rendered structure.
- Enhancement code should attach to explicit selectors and state attributes.
- Components that cannot provide meaningful no-JS behavior should not be part of the initial set.

This means the first wave should focus on components with strong native HTML foundations.

## v0 component scope

Build these first:

- button
- input
- textarea
- select
- field wrapper
- alert or notice
- container
- section
- stack
- top navigation

Delay these until the enhancement story is clearer:

- combobox
- popover
- tooltip
- menu button
- dialog
- custom select

The dividing line is simple: if a component is mostly HTML semantics plus styling, it belongs in the first wave. If it needs a client-side state machine to be credible, it should wait.

## API shape

Prefer small Rust APIs that map closely to HTML.

Good:

- `button(label)`
- `notice(flash)`
- `field(&field_state, control)`
- `container(content)`

Avoid:

- deeply nested builder APIs for simple markup
- generic configuration objects that hide the output structure
- component APIs that invent new concepts when HTML already has one

## Default style expectations

The first-party default style should eventually feel intentional and polished, but it should not drive the initial API.

The immediate goal is a styling contract that can support:

- a restrained, professional default theme
- app-level overrides with plain CSS
- alternate themes without forking component markup

## Asset integration

`ssw-html` should eventually make stylesheet and script inclusion easier, but component CSS should not depend on framework-specific magic in order to work.

That means:

- components should be styleable with a plain linked stylesheet
- future asset helpers should improve ergonomics, not define the only supported path

## Near-term roadmap

1. Stabilize the component styling contract.
2. Add stable class names and state attributes to the existing field and notice helpers.
   status: done in the current `ssw-components` helpers.
3. Create a small first-party default stylesheet, even if it is still rough.
   status: initial stylesheet added at `styles/ssw-components-default.css`.
4. Add the first layout primitives: container, section, stack.
   status: initial `container`, `section`, and `stack` helpers added.
5. Add simple form controls like button and select.
   status: initial button helpers added; select still pending.
6. Re-evaluate whether a separate theme crate should be created immediately or after an example app proves the CSS shape.

## Open questions

- Should the first default theme live in this repo as a crate, a plain CSS package, or example-app assets first?
- Should components expose slot-specific class names only, or both slot classes and helper attributes?
- How much of the eventual enhancement layer belongs in `ssw-components` versus a future dedicated enhancement crate?
- When `ssw-html` gets better asset helpers, should components expose any framework-aware convenience around them?

## Current recommendation

Do not try to build a large Base UI equivalent yet.

Do this first:

- define the styling contract
- keep the component set small
- build a thin first-party default CSS layer
- use a real example app to pressure the API

That is a much more solid foundation than copying advanced client-oriented primitives too early.
