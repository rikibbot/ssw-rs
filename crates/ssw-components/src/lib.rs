//! Optional UI components built on top of `ssw-html`.

use ssw_core::FlashMessage;
use ssw_html::{Markup, html, page as html_page};

/// Borrowed state for rendering a labeled form field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field<'a> {
    id: &'a str,
    name: &'a str,
    label: &'a str,
    value: &'a str,
    error: Option<&'a str>,
    required: bool,
}

impl<'a> Field<'a> {
    /// Creates a field with stable `id`, `name`, and label values.
    pub fn new(id: &'a str, name: &'a str, label: &'a str) -> Self {
        Self {
            id,
            name,
            label,
            value: "",
            error: None,
            required: false,
        }
    }

    /// Returns a copy with the submitted value attached.
    pub fn value(mut self, value: &'a str) -> Self {
        self.value = value;
        self
    }

    /// Returns a copy with the current validation error attached.
    pub fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }

    /// Returns a copy with the field marked as required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Returns the HTML id used by the control and label.
    pub fn id(&self) -> &str {
        self.id
    }

    /// Returns the submitted form name.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Returns the human-readable label.
    pub fn label(&self) -> &str {
        self.label
    }

    /// Returns the current submitted value.
    pub fn value_str(&self) -> &str {
        self.value
    }

    /// Returns the current validation error, if any.
    pub fn error_message(&self) -> Option<&str> {
        self.error
    }

    /// Returns whether the control is required.
    pub fn is_required(&self) -> bool {
        self.required
    }

    /// Returns the accessibility error id when the field is invalid.
    pub fn error_id(&self) -> Option<String> {
        self.error.map(|_| format!("{}-error", self.id))
    }

    /// Returns the `aria-describedby` value when the field is invalid.
    pub fn described_by(&self) -> Option<String> {
        self.error_id()
    }

    /// Returns the `aria-invalid` value for invalid controls.
    pub fn aria_invalid(&self) -> Option<&'static str> {
        self.error.map(|_| "true")
    }
}

/// Renders a simple status alert component.
pub fn alert(message: impl AsRef<str>) -> Markup {
    html! {
        div .ssw_alert role="status" {
            (message.as_ref())
        }
    }
}

/// Renders a flash-style notice using semantic level classes.
pub fn flash_notice(flash: &FlashMessage) -> Markup {
    let level = flash.level().as_str();

    html! {
        div class=(("notice", format!("notice-{level}"))) role="status" {
            p { (flash.message()) }
        }
    }
}

/// Renders a labeled field wrapper around a form control.
pub fn field(field: &Field<'_>, control: impl Into<Markup>) -> Markup {
    let error_id = field.error_id();
    let error = field.error_message();

    html! {
        div class=(("field", error.map(|_| "field-error"))) {
            label for=(field.id()) { (field.label()) }
            (control.into())
            @if error.is_some() {
                p id=(error_id.as_deref().unwrap()) .field_error {
                    (error.unwrap())
                }
            }
        }
    }
}

/// Renders a text input with label, error state, and accessibility wiring.
pub fn text_input(field_state: &Field<'_>) -> Markup {
    field(
        field_state,
        html! {
            input
                id=(field_state.id())
                type="text"
                name=(field_state.name())
                value=(field_state.value_str())
                required=(field_state.is_required())
                aria_invalid=(field_state.aria_invalid())
                aria_describedby=(field_state.described_by());
        },
    )
}

/// Renders an email input with label, error state, and accessibility wiring.
pub fn email_input(field_state: &Field<'_>) -> Markup {
    field(
        field_state,
        html! {
            input
                id=(field_state.id())
                type="email"
                name=(field_state.name())
                value=(field_state.value_str())
                required=(field_state.is_required())
                aria_invalid=(field_state.aria_invalid())
                aria_describedby=(field_state.described_by());
        },
    )
}

/// Renders a textarea with label, error state, and accessibility wiring.
pub fn textarea(field_state: &Field<'_>, rows: usize) -> Markup {
    field(
        field_state,
        html! {
            textarea
                id=(field_state.id())
                name=(field_state.name())
                rows=(rows)
                required=(field_state.is_required())
                aria_invalid=(field_state.aria_invalid())
                aria_describedby=(field_state.described_by()) {
                (field_state.value_str())
            }
        },
    )
}

/// Renders a hidden input, useful for CSRF tokens and method overrides.
pub fn hidden_input(name: impl AsRef<str>, value: impl AsRef<str>) -> Markup {
    html! {
        input
            type="hidden"
            name=(name.as_ref())
            value=(value.as_ref());
    }
}

/// Renders a full page using the `ssw-html` document builder.
pub fn page(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    html_page(title.as_ref()).body(body).render()
}

#[cfg(test)]
mod tests {
    use ssw_core::FlashMessage;

    use super::{Field, alert, email_input, flash_notice, hidden_input, text_input, textarea};

    #[test]
    fn alert_escapes_message() {
        let markup = alert("<unsafe>");

        assert!(markup.as_str().contains("&lt;unsafe&gt;"));
    }

    #[test]
    fn text_input_renders_error_state() {
        let field = Field::new("email", "email", "Email")
            .value("sprite-at-example.com")
            .error(Some("Email must look valid."))
            .required(true);

        let markup = email_input(&field);

        assert!(
            markup
                .as_str()
                .contains("<div class=\"field field-error\"><label for=\"email\">Email</label>")
        );
        assert!(
            markup
                .as_str()
                .contains("type=\"email\" name=\"email\" value=\"sprite-at-example.com\"")
        );
        assert!(markup.as_str().contains("required"));
        assert!(markup.as_str().contains("aria-invalid=\"true\""));
        assert!(markup.as_str().contains("aria-describedby=\"email-error\""));
        assert!(markup.as_str().contains("<p"));
        assert!(markup.as_str().contains("id=\"email-error\""));
        assert!(markup.as_str().contains("class=\"field-error\""));
        assert!(markup.as_str().contains("Email must look valid."));
    }

    #[test]
    fn textarea_preserves_value_without_error_markup() {
        let field = Field::new("message", "message", "Message").value("Hello");

        let markup = textarea(&field, 4);

        assert!(
            markup.as_str().contains(
                "<label for=\"message\">Message</label><textarea id=\"message\" name=\"message\" rows=\"4\">Hello</textarea>"
            )
        );
        assert!(!markup.as_str().contains("aria-invalid"));
        assert!(!markup.as_str().contains("field-error"));
    }

    #[test]
    fn text_input_escapes_submitted_value() {
        let field = Field::new("name", "name", "Name").value("<unsafe>");

        let markup = text_input(&field);

        assert!(markup.as_str().contains("value=\"&lt;unsafe&gt;\""));
    }

    #[test]
    fn flash_notice_uses_semantic_level_classes() {
        let markup = flash_notice(&FlashMessage::success("Saved"));

        assert!(markup.as_str().contains("class=\"notice notice-success\""));
        assert!(markup.as_str().contains("Saved"));
    }

    #[test]
    fn hidden_input_renders_hidden_control() {
        let markup = hidden_input("csrf_token", "abc123");

        assert!(
            markup
                .as_str()
                .contains("type=\"hidden\" name=\"csrf_token\" value=\"abc123\"")
        );
    }
}
