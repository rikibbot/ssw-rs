use ssw_html::{html, Markup};

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

/// A single breadcrumb item for hierarchical page context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreadcrumbItem<'a> {
    href: Option<&'a str>,
    label: &'a str,
    current: bool,
}

/// A single pagination item for numbered page navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationItem<'a> {
    href: Option<&'a str>,
    label: &'a str,
    current: bool,
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

impl<'a> BreadcrumbItem<'a> {
    /// Creates a linked breadcrumb item.
    pub fn link(href: &'a str, label: &'a str) -> Self {
        Self {
            href: Some(href),
            label,
            current: false,
        }
    }

    /// Creates the current breadcrumb item.
    pub fn current(label: &'a str) -> Self {
        Self {
            href: None,
            label,
            current: true,
        }
    }

    /// Returns the breadcrumb target, when present.
    pub fn href(&self) -> Option<&str> {
        self.href
    }

    /// Returns the visible label.
    pub fn label(&self) -> &str {
        self.label
    }

    /// Returns whether the breadcrumb item is the current page.
    pub fn is_current(&self) -> bool {
        self.current
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

impl<'a> PaginationItem<'a> {
    /// Creates a linked pagination item.
    pub fn link(href: &'a str, label: &'a str) -> Self {
        Self {
            href: Some(href),
            label,
            current: false,
        }
    }

    /// Creates the current pagination item.
    pub fn current(label: &'a str) -> Self {
        Self {
            href: None,
            label,
            current: true,
        }
    }

    /// Returns the pagination target, when present.
    pub fn href(&self) -> Option<&str> {
        self.href
    }

    /// Returns the visible label.
    pub fn label(&self) -> &str {
        self.label
    }

    /// Returns whether the item represents the current page.
    pub fn is_current(&self) -> bool {
        self.current
    }
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

/// Renders breadcrumb navigation for the current page context.
pub fn breadcrumbs(items: &[BreadcrumbItem<'_>]) -> Markup {
    html! {
        nav class="ssw-breadcrumbs" aria_label="Breadcrumb" {
            ol class="ssw-breadcrumbs__list" {
                @for item in items {
                    li class="ssw-breadcrumbs__item" {
                        ({
                            if item.is_current() {
                                html! {
                                    span class="ssw-breadcrumbs__current" aria_current="page" {
                                        (item.label())
                                    }
                                }
                            } else {
                                html! {
                                    a class="ssw-breadcrumbs__link" href=(item.href().unwrap_or("#")) {
                                        (item.label())
                                    }
                                }
                            }
                        })
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
    let actions_markup = match actions {
        Some(actions) => html! {
            div class="ssw-page-header__actions" {
                (actions)
            }
        },
        None => Markup::new(),
    };

    html! {
        header class="ssw-page-header" {
            p class="ssw-page-header__eyebrow" { (eyebrow.as_ref()) }
            h1 class="ssw-page-header__title" { (title.as_ref()) }
            div class="ssw-page-header__body" {
                (body.into())
            }
            (actions_markup)
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
    let actions_markup = match actions {
        Some(actions) => html! {
            div class="ssw-empty-state__actions" {
                (actions)
            }
        },
        None => Markup::new(),
    };

    html! {
        section class="ssw-empty-state" {
            div class="ssw-empty-state__body" {
                h2 class="ssw-empty-state__title" { (title.as_ref()) }
                div class="ssw-empty-state__copy" {
                    (body.into())
                }
            }
            (actions_markup)
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

/// Renders a simple pagination navigation row.
pub fn pagination(items: &[PaginationItem<'_>]) -> Markup {
    html! {
        nav class="ssw-pagination" aria_label="Pagination" {
            ol class="ssw-pagination__list" {
                @for item in items {
                    li class="ssw-pagination__item" {
                        ({
                            if item.is_current() {
                                html! {
                                    span class="ssw-pagination__current" aria_current="page" {
                                        (item.label())
                                    }
                                }
                            } else {
                                html! {
                                    a class="ssw-pagination__link" href=(item.href().unwrap_or("#")) {
                                        (item.label())
                                    }
                                }
                            }
                        })
                    }
                }
            }
        }
    }
}
