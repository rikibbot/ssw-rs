//! App-owned components for the projects demo.
//!
//! This module exists to show the intended `ssw-rs` component authoring model:
//! - framework primitives live in `ssw-components`
//! - app-specific components are ordinary Rust functions returning `Markup`
//! - `ssw-css` is optional and local to the component that needs it

use ssw_components::{
    BadgeVariant, MetaItem, StatItem, badge_with_variant, card_header, meta_list, page_actions,
    section, stack, stat_list,
};
use ssw_css::{StyleSheet, css};
use ssw_html::{Markup, html};

/// Returns the local stylesheet used by the projects demo app-owned components.
pub fn project_styles() -> StyleSheet {
    css! {
        ".card" {
            display: grid;
            gap: 0.65 rem;
            padding: 1 rem;
            border: 1 px solid var(--ssw-color-border);
            border-radius: var(--ssw-radius-md);
            background: color-mix(in srgb, var(--ssw-color-surface) 84%, white);
            text-decoration: none;
            transition: border-color 160 ms ease, background-color 160 ms ease, transform 160 ms ease;
        }

        ".card:hover" {
            border-color: var(--ssw-color-border-strong);
            background: var(--ssw-color-surface);
            transform: translateY(-1 px);
        }

        ".card-head" {
            display: flex;
            flex-wrap: wrap;
            align-items: center;
            justify-content: space-between;
            gap: 0.65 rem;
        }

        ".eyebrow" {
            margin: 0;
            color: var(--ssw-color-text-muted);
            font-size: 0.75 rem;
            font-weight: 600;
            letter-spacing: 0.08 em;
            text-transform: uppercase;
        }

        ".title" {
            margin: 0;
            color: var(--ssw-color-text);
            font-size: 1.05 rem;
            font-weight: 600;
            line-height: 1.15;
            letter-spacing: -0.02 em;
        }

        ".summary" {
            margin: 0;
            color: #52525b;
            font-size: 0.95 rem;
            line-height: 1.6;
        }

        ".copy-group" {
            display: grid;
            gap: 0.9 rem;
        }

        ".copy" {
            margin: 0;
            color: #52525b;
            font-size: 0.95 rem;
            line-height: 1.6;
        }

        ".form" {
            display: grid;
            gap: 0.9 rem;
        }
    }
}

/// Renders the app-owned status badge used by the projects demo.
pub fn project_status_badge(status: &str) -> Markup {
    let variant = match status {
        "active" => BadgeVariant::Success,
        "review" => BadgeVariant::Info,
        _ => BadgeVariant::Neutral,
    };

    badge_with_variant(status, variant)
}

/// Renders the app-owned project-card component used in the project list.
pub fn project_card(
    styles: &StyleSheet,
    href: impl AsRef<str>,
    client: impl AsRef<str>,
    title: impl AsRef<str>,
    summary: impl AsRef<str>,
    status: &str,
) -> Markup {
    html! {
        a class=(styles.class("card")) href=(href.as_ref()) {
            div class=(styles.class("card-head")) {
                p class=(styles.class("eyebrow")) { (client.as_ref()) }
                (project_status_badge(status))
            }
            h2 class=(styles.class("title")) { (title.as_ref()) }
            p class=(styles.class("summary")) { (summary.as_ref()) }
        }
    }
}

/// Renders an app-owned metadata panel composed from framework primitives.
pub fn project_metadata_panel(stats: &[StatItem<'_>], details: &[MetaItem<'_>]) -> Markup {
    let details_markup = if details.is_empty() {
        Markup::new()
    } else {
        html! { (meta_list(details)) }
    };

    section(stack(html! {
        (card_header("Project metadata", html! {
            p { "This is intentionally boring data, because the layout should still feel deliberate." }
        }))
        (stat_list(stats))
        (details_markup)
    }))
}

/// Renders an app-owned narrative panel for project detail and edit routes.
pub fn project_story_panel(
    styles: &StyleSheet,
    title: &str,
    intro: impl Into<Markup>,
    paragraphs: &[&str],
    actions: Option<Markup>,
) -> Markup {
    let actions_markup = actions.map(page_actions).unwrap_or_default();

    section(stack(html! {
        (card_header(title, intro.into()))
        div class=(styles.class("copy-group")) {
            @for paragraph in paragraphs {
                p class=(styles.class("copy")) { (paragraph) }
            }
        }
        (actions_markup)
    }))
}
