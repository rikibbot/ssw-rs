use ssw_html::{Markup, html};

/// The supported visual variants for button components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// The default emphasized button treatment.
    Primary,
    /// A quieter secondary treatment.
    Secondary,
}

impl ButtonVariant {
    /// Returns a stable lowercase variant identifier for styling hooks.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
        }
    }
}

/// Renders a button with the default primary styling.
pub fn button(label: impl AsRef<str>) -> Markup {
    button_with_variant(label, ButtonVariant::Primary)
}

/// Renders a button with an explicit visual variant.
pub fn button_with_variant(label: impl AsRef<str>, variant: ButtonVariant) -> Markup {
    let variant_name = variant.as_str();

    html! {
        button class="ssw-button" data_variant=(variant_name) type="button" {
            (label.as_ref())
        }
    }
}

/// Renders a submit button with the default primary styling.
pub fn submit_button(label: impl AsRef<str>) -> Markup {
    html! {
        button class="ssw-button" data_variant="primary" type="submit" {
            (label.as_ref())
        }
    }
}

/// Renders a link-shaped action for page headers and inline action rows.
pub fn link_button(href: impl AsRef<str>, label: impl AsRef<str>) -> Markup {
    html! {
        a class="ssw-link-button" href=(href.as_ref()) {
            (label.as_ref())
        }
    }
}
