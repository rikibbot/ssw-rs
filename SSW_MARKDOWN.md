# ssw-markdown Design

## Purpose

`ssw-markdown` should make Markdown a first-class content format for `ssw-rs`.

The goal is to render Markdown into `ssw-html::Markup` while preserving the framework model:

- server-rendered output
- predictable escaping
- no client runtime requirement
- optional app-owned components
- optional `ssw-css` styling

This crate should be useful for documentation pages first. Static site generation can build on top later, but it should not be the first abstraction.

## Goals

- Convert Markdown strings into `ssw-html::Markup`.
- Provide a safe default rendering mode.
- Support documentation features such as heading ids and table of contents extraction.
- Allow Markdown authors to reference app-registered server-rendered components.
- Allow Markdown pages and Markdown components to collect optional `ssw-css` styles.
- Keep output as plain HTML and plain CSS.

## Non-goals

- Writing a Markdown parser from scratch.
- Executing arbitrary Rust from Markdown.
- Replacing `ssw-html` for application UI.
- Building a full static site generator in the first version.
- Requiring `ssw-components` or `ssw-css`.
- Client-side hydration or MDX-style runtime assumptions.

## Crate boundary

Proposed crate:

- `ssw-markdown`

Responsibilities:

- Markdown parsing integration
- Markdown-to-`Markup` rendering
- safe raw HTML handling
- component directive parsing and dispatch
- heading id and table of contents collection
- optional style collection for Markdown typography and component-local styles

It should depend on `ssw-html`.

It may optionally integrate with `ssw-css`, but plain Markdown rendering should work without scoped CSS.

It should not depend on Actix, Workers, or any HTTP backend.

## Rendering model

The public output should be richer than a bare `Markup` value.

Recommended shape:

```rust
pub struct RenderedMarkdown {
    pub body: Markup,
    pub head: Markup,
    pub styles: Vec<StyleSheet>,
    pub toc: TableOfContents,
}
```

Exact field visibility can change, but the concept matters: Markdown rendering may produce body HTML, page-level assets, collected scoped styles, and document metadata.

A simple helper can still exist:

```rust
let body = ssw_markdown::render(markdown);
```

But the main API should make room for real documentation pages:

```rust
let rendered = Markdown::new(source)
    .heading_ids(true)
    .table_of_contents(true)
    .render();
```

## Safety model

Markdown must be safe by default.

Rules:

- Markdown text is escaped correctly.
- Raw HTML inside Markdown is disabled or escaped by default.
- Allowing raw HTML requires an explicit trusted-content option.
- Component directives are resolved through an app-owned registry, not by executing code from Markdown.

Recommended API:

```rust
let rendered = Markdown::new(source)
    .allow_raw_html(false)
    .render();
```

Trusted local docs may opt in:

```rust
let rendered = Markdown::new(source)
    .allow_raw_html(true)
    .render();
```

This distinction matters because `ssw-html::Markup::raw(...)` is intentionally trusted HTML.

## Component directives

Markdown should be able to reference server-rendered components without becoming a second programming language.

Recommended syntax:

```markdown
# Deploying

Regular Markdown text.

:::callout level="warning" title="Before deploying"
Make sure the Worker environment has the expected bindings.
:::

:::status-badge status="active" :::
```

Rust code registers the components:

```rust
let rendered = Markdown::new(source)
    .component("callout", callout_component)
    .component("status-badge", status_badge_component)
    .render();
```

Component functions should receive structured input:

- directive name
- attributes
- rendered Markdown body
- source span, if available
- render context for collecting styles or diagnostics

The Markdown file should not import Rust modules or run arbitrary expressions. It names components; Rust owns behavior.

## Component body rendering

Directive bodies should usually be rendered as nested Markdown before they reach the component.

Example:

```markdown
:::callout title="Note"
This body can include **Markdown** and links.
:::
```

The component receives body `Markup`, not raw Markdown text, unless it explicitly asks for raw text.

This keeps authoring ergonomic while preserving clear rendering boundaries.

## Markdown typography and `ssw-css`

Documentation sites need a good default reading experience.

`ssw-markdown` should support a Markdown typography stylesheet that is opt-in and replaceable.

Recommended direction:

- unstyled Markdown output by default, with stable classes
- optional first-party Markdown typography stylesheet
- optional `ssw-css` helper for app-owned documentation themes

Generated Markdown should expose a stable root class:

```html
<article class="ssw-markdown">
  ...
</article>
```

The default typography layer can target that root:

```css
.ssw-markdown h1 { ... }
.ssw-markdown pre { ... }
.ssw-markdown table { ... }
```

For app-owned styling, `ssw-css` can provide scoped styles:

```rust
let prose = css! {
    ".root" {
        display: grid;
        gap: 1.25 rem;
    }

    ".root h2" {
        margin-top: 2 rem;
    }
};

let rendered = Markdown::new(source)
    .root_class(prose.class("root"))
    .render();
```

The browser still receives normal HTML and CSS. There is no runtime style injection requirement.

## Style collection

Component directives may need styles.

For example, a documentation callout component may render with a local `ssw-css` stylesheet. The renderer should be able to collect those styles while rendering the Markdown page.

Possible shape:

```rust
fn callout_component(input: ComponentInput, ctx: &mut MarkdownContext) -> Markup {
    let styles = callout_styles();
    ctx.add_style(styles.clone());

    html! {
        aside class=(styles.class("root")) {
            (input.body)
        }
    }
}
```

The exact API should avoid unnecessary cloning if possible, but the direction is important:

- components render body markup
- components can register styles
- the final page can place collected styles in the document head or a future extracted asset

## Diagnostics

Component directives need clear failure behavior.

Recommended defaults:

- unknown component names produce a structured render error
- invalid attributes produce a structured render error
- docs examples can choose to render visible diagnostic markup in development

Do not silently drop unknown directives. Documentation systems become hard to trust when broken component calls disappear.

## Static site generation

Static site generation should be built later and on top.

Likely future crate or tool:

- `ssw-site`

Responsibilities:

- file discovery
- frontmatter parsing
- route generation
- asset copying
- static HTML output
- optional sitemap and feed generation

`ssw-markdown` should stay focused on rendering Markdown content into `ssw-rs` page building blocks.

## v0 milestone

The first useful milestone should be small:

1. Add `ssw-markdown`.
2. Render Markdown strings into safe `Markup`.
3. Wrap output in a stable `.ssw-markdown` root.
4. Support heading ids.
5. Support one component directive shape.
6. Support a component registry.
7. Prove it with a docs example route.

Only after that should the project consider table of contents, syntax highlighting, frontmatter, or static generation.

## Open questions

- Which Markdown parser should be used.
- Whether directive syntax should follow an existing convention or stay `ssw-rs` specific.
- How much attribute parsing should be typed in v0.
- Whether style collection belongs directly in `RenderedMarkdown` or in a shared asset collection type.
- Whether syntax highlighting should be plain class emission first, with highlighter integration behind an optional feature later.
