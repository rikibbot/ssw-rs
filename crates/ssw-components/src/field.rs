use ssw_html::{Markup, html};

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

/// Renders a labeled field wrapper around a form control.
pub fn field(field: &Field<'_>, control: impl Into<Markup>) -> Markup {
    let error_markup = match (field.error_id(), field.error_message()) {
        (Some(error_id), Some(error)) => html! {
            p id=(error_id) class="ssw-field__error" {
                (error)
            }
        },
        _ => Markup::new(),
    };

    html! {
        div class="ssw-field" data_invalid=(field.data_invalid()) {
            label class="ssw-field__label" for=(field.id()) { (field.label()) }
            (control.into())
            (error_markup)
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
