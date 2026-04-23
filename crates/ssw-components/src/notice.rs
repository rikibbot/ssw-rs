use ssw_core::{FlashLevel, FlashMessage};
use ssw_html::{Markup, html};

fn notice_role(level: FlashLevel) -> &'static str {
    match level {
        FlashLevel::Info | FlashLevel::Success => "status",
        FlashLevel::Warning | FlashLevel::Error => "alert",
    }
}

/// Renders a simple informational notice component.
pub fn alert(message: impl AsRef<str>) -> Markup {
    html! {
        div class=(("ssw-notice", "ssw-notice--info")) data_level="info" role=(notice_role(FlashLevel::Info)) {
            p class="ssw-notice__message" {
                (message.as_ref())
            }
        }
    }
}

/// Renders a flash-style notice using semantic level classes.
pub fn flash_notice(flash: &FlashMessage) -> Markup {
    let level = flash.level().as_str();
    let level_class = format!("ssw-notice--{level}");
    let role = notice_role(flash.level());

    html! {
        div class=(("ssw-notice", level_class.as_str())) data_level=(level) role=(role) {
            p class="ssw-notice__message" { (flash.message()) }
        }
    }
}

/// A single validation-summary row, optionally linking to a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationItem<'a> {
    href: Option<&'a str>,
    message: &'a str,
}

impl<'a> ValidationItem<'a> {
    /// Creates a validation item with message text only.
    pub fn new(message: &'a str) -> Self {
        Self {
            href: None,
            message,
        }
    }

    /// Creates a validation item that links to a field or fragment target.
    pub fn link(href: &'a str, message: &'a str) -> Self {
        Self {
            href: Some(href),
            message,
        }
    }

    /// Returns the optional link target.
    pub fn href(&self) -> Option<&str> {
        self.href
    }

    /// Returns the visible validation message.
    pub fn message(&self) -> &str {
        self.message
    }
}

/// Renders a validation summary notice with optional linked field errors.
pub fn validation_summary(summary: impl AsRef<str>, items: &[ValidationItem<'_>]) -> Markup {
    let flash = FlashMessage::error(summary.as_ref());

    html! {
        div
            class=(["ssw-notice", "ssw-notice--error", "ssw-validation-summary"])
            data_level="error"
            role=(notice_role(FlashLevel::Error)) {
            p class="ssw-notice__message" {
                (flash.message())
            }
            @if !items.is_empty() {
                ul class="ssw-validation-summary__list" {
                    @for item in items {
                        li class="ssw-validation-summary__item" {
                            @if item.href().is_some() {
                                a class="ssw-validation-summary__link" href=(item.href().unwrap()) {
                                    (item.message())
                                }
                            }
                            @if item.href().is_none() {
                                span class="ssw-validation-summary__text" {
                                    (item.message())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
