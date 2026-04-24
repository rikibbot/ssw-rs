# Custom Components

`ssw-rs` should make custom component authoring feel ordinary.

The framework owns:

- HTML authoring in Rust
- stable unstyled primitives in `ssw-components`
- optional scoped local styling in `ssw-css`

Your app should own:

- product-specific components
- local naming and composition
- aesthetic decisions that do not belong in the framework

## Recommended model

Build custom components as plain Rust functions returning `Markup`.

Start from framework primitives when they help:

- `section`
- `stack`
- `card_header`
- `badge`
- `stat_list`
- `meta_list`
- `page_header`

Then add app-owned composition around them.

Example shape:

```rust
use ssw_components::{card_header, section, stack};
use ssw_html::{Markup, html};

fn project_panel(title: &str, body: Markup) -> Markup {
    section(stack(html! {
        (card_header(title, Markup::new()))
        (body)
    }))
}
```

This keeps component authoring explicit and inspectable. There is no special component runtime.

## Styling

Use plain CSS when styles are naturally shared across the app.

Use `ssw-css` when styles are local to one component or one page fragment and keeping them next to the component makes the code clearer.

That means:

- `ssw-components` stays independent from `ssw-css`
- app code can opt into scoped local styles where they help
- the browser still receives normal CSS and normal class names

## What belongs in `ssw-components`

Promote a component into `ssw-components` only when it is:

- clearly reusable across multiple apps or example flows
- structurally stable
- honest without client-side runtime tricks
- better as a framework primitive than as app-owned composition

Do not move product-specific UI into the framework just because it looks polished.

## Example in this repo

See [`examples/ssw-projects-demo/src/components.rs`](./examples/ssw-projects-demo/src/components.rs).

That module shows the intended split:

- `project_card` is app-owned and styled locally with `ssw-css`
- `project_status_badge` is app-owned composition over `badge_with_variant`
- `project_metadata_panel` is app-owned composition over `section`, `card_header`, `stat_list`, and `meta_list`
- `project_story_panel` is an app-owned narrative panel used by detail, edit, and 404 routes, with local `ssw-css` copy and form-adjacent styles

That is the standard `ssw-rs` should optimize for: strong primitives, easy composition, and optional local styling without framework magic.
