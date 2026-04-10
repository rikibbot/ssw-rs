//! Optional UI components built on top of `ssw-html`.

use ssw_core::FlashMessage;
use ssw_html::{Markup, html, page as html_page};

/// The supported visual variants for button components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// The default emphasized button treatment.
    Primary,
    /// A quieter secondary treatment.
    Secondary,
}

fn notice_role(level: ssw_core::FlashLevel) -> &'static str {
    match level {
        ssw_core::FlashLevel::Info | ssw_core::FlashLevel::Success => "status",
        ssw_core::FlashLevel::Warning | ssw_core::FlashLevel::Error => "alert",
    }
}

/// A single option for the native select helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectOption<'a> {
    value: &'a str,
    label: &'a str,
}

impl<'a> SelectOption<'a> {
    /// Creates a select option with a form value and visible label.
    pub fn new(value: &'a str, label: &'a str) -> Self {
        Self { value, label }
    }

    /// Returns the submitted value for the option.
    pub fn value(&self) -> &str {
        self.value
    }

    /// Returns the visible label for the option.
    pub fn label(&self) -> &str {
        self.label
    }
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

    /// Returns a semantic invalid-state marker for styling hooks.
    pub fn data_invalid(&self) -> Option<&'static str> {
        self.error.map(|_| "true")
    }
}

/// Renders a simple status alert component.
pub fn alert(message: impl AsRef<str>) -> Markup {
    html! {
        div class=(("ssw-notice", "ssw-notice--info")) data_level="info" role=(notice_role(ssw_core::FlashLevel::Info)) {
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

/// Renders a labeled field wrapper around a form control.
pub fn field(field: &Field<'_>, control: impl Into<Markup>) -> Markup {
    let error_id = field.error_id();
    let error = field.error_message();
    let invalid = field.data_invalid();

    html! {
        div class="ssw-field" data_invalid=(invalid) {
            label class="ssw-field__label" for=(field.id()) { (field.label()) }
            (control.into())
            @if error.is_some() {
                p id=(error_id.as_deref().unwrap()) class="ssw-field__error" {
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
                class="ssw-input"
                data_invalid=(field_state.data_invalid())
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
                class="ssw-input"
                data_invalid=(field_state.data_invalid())
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
                class="ssw-textarea"
                data_invalid=(field_state.data_invalid())
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

/// Renders a native select control with label, options, and error wiring.
pub fn select(field_state: &Field<'_>, options: &[SelectOption<'_>]) -> Markup {
    field(
        field_state,
        html! {
            select
                class="ssw-select"
                data_invalid=(field_state.data_invalid())
                id=(field_state.id())
                name=(field_state.name())
                required=(field_state.is_required())
                aria_invalid=(field_state.aria_invalid())
                aria_describedby=(field_state.described_by()) {
                @for option in options {
                    option value=(option.value()) selected=(field_state.value_str() == option.value()) {
                        (option.label())
                    }
                }
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
    use ssw_html::Markup;

    use super::{
        ButtonVariant, Field, SelectOption, alert, button, button_with_variant, container,
        email_input, flash_notice, hidden_input, section, select, stack, submit_button, text_input,
        textarea,
    };

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
                .contains("<div class=\"ssw-field\" data-invalid=\"true\"><label class=\"ssw-field__label\" for=\"email\">Email</label>")
        );
        assert!(
            markup
                .as_str()
                .contains("class=\"ssw-input\" data-invalid=\"true\" id=\"email\" type=\"email\" name=\"email\" value=\"sprite-at-example.com\"")
        );
        assert!(markup.as_str().contains("required"));
        assert!(markup.as_str().contains("aria-invalid=\"true\""));
        assert!(markup.as_str().contains("aria-describedby=\"email-error\""));
        assert!(markup.as_str().contains("<p"));
        assert!(markup.as_str().contains("id=\"email-error\""));
        assert!(markup.as_str().contains("class=\"ssw-field__error\""));
        assert!(markup.as_str().contains("Email must look valid."));
    }

    #[test]
    fn textarea_preserves_value_without_error_markup() {
        let field = Field::new("message", "message", "Message").value("Hello");

        let markup = textarea(&field, 4);

        assert!(
            markup.as_str().contains(
                "<label class=\"ssw-field__label\" for=\"message\">Message</label><textarea class=\"ssw-textarea\" id=\"message\" name=\"message\" rows=\"4\">Hello</textarea>"
            )
        );
        assert!(!markup.as_str().contains("aria-invalid"));
        assert!(!markup.as_str().contains("data-invalid"));
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

        assert!(markup.as_str().contains(
            "class=\"ssw-notice ssw-notice--success\" data-level=\"success\" role=\"status\""
        ));
        assert!(markup.as_str().contains("class=\"ssw-notice__message\""));
        assert!(markup.as_str().contains("Saved"));
    }

    #[test]
    fn error_flash_notice_uses_alert_role() {
        let markup = flash_notice(&FlashMessage::error("Failed"));

        assert!(
            markup
                .as_str()
                .contains("data-level=\"error\" role=\"alert\"")
        );
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

    #[test]
    fn button_uses_default_primary_variant() {
        let markup = button("Save");

        assert!(markup.as_str().contains(
            "<button class=\"ssw-button\" data-variant=\"primary\" type=\"button\">Save</button>"
        ));
    }

    #[test]
    fn button_supports_secondary_variant() {
        let markup = button_with_variant("Cancel", ButtonVariant::Secondary);

        assert!(markup.as_str().contains("data-variant=\"secondary\""));
        assert!(markup.as_str().contains("Cancel"));
    }

    #[test]
    fn submit_button_sets_submit_type() {
        let markup = submit_button("Send");

        assert!(markup.as_str().contains("type=\"submit\""));
        assert!(markup.as_str().contains("Send"));
    }

    #[test]
    fn layout_primitives_wrap_content_with_stable_classes() {
        let markup = container(section(stack(Markup::text("Hello"))));

        assert!(markup.as_str().contains("<div class=\"ssw-container\">"));
        assert!(markup.as_str().contains("<section class=\"ssw-section\">"));
        assert!(
            markup
                .as_str()
                .contains("<div class=\"ssw-stack\">Hello</div>")
        );
    }

    #[test]
    fn layout_primitives_escape_plain_text_content() {
        let markup = container("<unsafe>");

        assert!(markup.as_str().contains("&lt;unsafe&gt;"));
        assert!(!markup.as_str().contains("<unsafe>"));
    }

    #[test]
    fn select_marks_current_option_and_uses_stable_classes() {
        let field = Field::new("topic", "topic", "Topic")
            .value("support")
            .required(true);
        let options = [
            SelectOption::new("", "Choose a topic"),
            SelectOption::new("support", "Support"),
            SelectOption::new("sales", "Sales"),
        ];

        let markup = select(&field, &options);

        assert!(markup.as_str().contains("class=\"ssw-select\""));
        assert!(
            markup
                .as_str()
                .contains("<option value=\"support\" selected>Support</option>")
        );
        assert!(
            markup
                .as_str()
                .contains("<label class=\"ssw-field__label\" for=\"topic\">Topic</label>")
        );
    }
}
