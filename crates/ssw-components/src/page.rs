use ssw_html::{Markup, html, page as html_page};

/// Renders a full page using the `ssw-html` document builder.
pub fn page(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    html_page(title.as_ref()).body(body).render()
}

/// Renders a vertical shell for top-level page composition.
pub fn page_shell(content: impl Into<Markup>) -> Markup {
    html! {
        div class="ssw-page-shell" {
            (content.into())
        }
    }
}

/// Renders a page header with optional actions.
pub fn page_header(
    eyebrow: impl AsRef<str>,
    title: impl AsRef<str>,
    body: impl Into<Markup>,
    actions: Option<Markup>,
) -> Markup {
    html! {
        header class="ssw-page-header" {
            p class="ssw-page-header__eyebrow" { (eyebrow.as_ref()) }
            h1 class="ssw-page-header__title" { (title.as_ref()) }
            div class="ssw-page-header__body" {
                (body.into())
            }
            @if actions.is_some() {
                div class="ssw-page-header__actions" {
                    (actions.unwrap())
                }
            }
        }
    }
}

/// Renders a simple action row for links and buttons that live near a page header.
pub fn page_actions(content: impl Into<Markup>) -> Markup {
    html! {
        div class="ssw-page-actions" {
            (content.into())
        }
    }
}

/// Renders a heading block for a card or section surface.
pub fn card_header(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    html! {
        div class="ssw-card-header" {
            h2 class="ssw-card-header__title" { (title.as_ref()) }
            div class="ssw-card-header__body" {
                (body.into())
            }
        }
    }
}
