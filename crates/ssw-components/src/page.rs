use ssw_html::{Markup, html, page as html_page};

/// A navigation item for a simple top-level app navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavItem<'a> {
    href: &'a str,
    label: &'a str,
    current: bool,
}

/// A single labeled row for compact project or entity metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetaItem<'a> {
    label: &'a str,
    value: &'a str,
}

impl<'a> MetaItem<'a> {
    /// Creates a metadata row with a label and plain-text value.
    pub fn new(label: &'a str, value: &'a str) -> Self {
        Self { label, value }
    }

    /// Returns the row label.
    pub fn label(&self) -> &str {
        self.label
    }

    /// Returns the row value.
    pub fn value(&self) -> &str {
        self.value
    }
}

impl<'a> NavItem<'a> {
    /// Creates a navigation item with an href and visible label.
    pub fn new(href: &'a str, label: &'a str) -> Self {
        Self {
            href,
            label,
            current: false,
        }
    }

    /// Marks the item as the current page.
    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// Returns the navigation target.
    pub fn href(&self) -> &str {
        self.href
    }

    /// Returns the visible label.
    pub fn label(&self) -> &str {
        self.label
    }

    /// Returns whether this item represents the current page.
    pub fn is_current(&self) -> bool {
        self.current
    }
}

/// Renders a full page using the `ssw-html` document builder.
pub fn page(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    html_page(title.as_ref()).body(body).render()
}

/// Renders a simple top navigation bar with a brand link and current-page state.
pub fn top_nav(
    brand_href: impl AsRef<str>,
    brand_label: impl AsRef<str>,
    items: &[NavItem<'_>],
) -> Markup {
    html! {
        nav class="ssw-top-nav" aria_label="Primary" {
            a class="ssw-top-nav__brand" href=(brand_href.as_ref()) {
                (brand_label.as_ref())
            }
            ul class="ssw-top-nav__list" {
                @for item in items {
                    li class="ssw-top-nav__item" {
                        a
                            class="ssw-top-nav__link"
                            href=(item.href())
                            data_current=(item.is_current().then_some("true"))
                            aria_current=(item.is_current().then_some("page")) {
                            (item.label())
                        }
                    }
                }
            }
        }
    }
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

/// Renders an empty-state block with optional actions.
pub fn empty_state(
    title: impl AsRef<str>,
    body: impl Into<Markup>,
    actions: Option<Markup>,
) -> Markup {
    html! {
        section class="ssw-empty-state" {
            div class="ssw-empty-state__body" {
                h2 class="ssw-empty-state__title" { (title.as_ref()) }
                div class="ssw-empty-state__copy" {
                    (body.into())
                }
            }
            @if actions.is_some() {
                div class="ssw-empty-state__actions" {
                    (actions.unwrap())
                }
            }
        }
    }
}

/// Renders a simple labeled metadata list.
pub fn meta_list(items: &[MetaItem<'_>]) -> Markup {
    html! {
        dl class="ssw-meta-list" {
            @for item in items {
                div class="ssw-meta-list__row" {
                    dt class="ssw-meta-list__label" { (item.label()) }
                    dd class="ssw-meta-list__value" { (item.value()) }
                }
            }
        }
    }
}
