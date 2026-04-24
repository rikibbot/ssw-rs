use ssw_html::{Markup, html};

/// Renders a constrained layout container.
pub fn container(content: impl Into<Markup>) -> Markup {
    html! {
        div class="ssw-container" {
            (content.into())
        }
    }
}

/// Renders a section wrapper for page composition.
pub fn section(content: impl Into<Markup>) -> Markup {
    html! {
        section class="ssw-section" {
            (content.into())
        }
    }
}

/// Renders a vertical stack wrapper for grouped content.
pub fn stack(content: impl Into<Markup>) -> Markup {
    html! {
        div class="ssw-stack" {
            (content.into())
        }
    }
}
