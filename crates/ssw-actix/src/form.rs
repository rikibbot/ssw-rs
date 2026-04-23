use std::collections::HashMap;

use actix_web::{HttpRequest, web};
use ssw_core::CsrfError;

use crate::{RequestContext, request_context};

/// Request form data with small helpers for common server-side access patterns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FormData {
    fields: HashMap<String, String>,
}

impl FormData {
    /// Builds form data from a field map.
    pub fn new(fields: HashMap<String, String>) -> Self {
        Self { fields }
    }

    /// Returns the submitted value for a field.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// Returns a field value or the empty string when the field is missing.
    pub fn value(&self, name: &str) -> String {
        self.get(name).unwrap_or_default().to_owned()
    }

    /// Returns a field value or a provided default when the field is missing.
    pub fn value_or(&self, name: &str, default: &str) -> String {
        self.get(name).unwrap_or(default).to_owned()
    }

    /// Returns the underlying submitted fields.
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.fields
    }

    /// Consumes the form data and returns the underlying field map.
    pub fn into_inner(self) -> HashMap<String, String> {
        self.fields
    }
}

impl From<HashMap<String, String>> for FormData {
    fn from(fields: HashMap<String, String>) -> Self {
        Self::new(fields)
    }
}

impl From<web::Form<HashMap<String, String>>> for FormData {
    fn from(form: web::Form<HashMap<String, String>>) -> Self {
        Self::new(form.into_inner())
    }
}

/// A submitted Actix form paired with the current request context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSubmission {
    context: RequestContext,
    data: FormData,
}

impl FormSubmission {
    /// Builds a submitted form from an Actix request and extractor payload.
    pub fn from_request(request: &HttpRequest, form: web::Form<HashMap<String, String>>) -> Self {
        Self {
            context: request_context(request),
            data: FormData::from(form),
        }
    }

    /// Returns the submitted data.
    pub fn data(&self) -> &FormData {
        &self.data
    }

    /// Returns the request context associated with the form.
    pub fn context(&self) -> &RequestContext {
        &self.context
    }

    /// Verifies the submitted CSRF token and returns a verified form on success.
    pub fn verify_csrf(self) -> Result<VerifiedForm, InvalidForm> {
        match self
            .context
            .verify_csrf(self.data.get(ssw_core::CSRF_FORM_FIELD))
        {
            Ok(()) => Ok(VerifiedForm {
                context: self.context,
                data: self.data,
            }),
            Err(error) => Err(InvalidForm {
                context: self.context,
                data: self.data,
                error,
            }),
        }
    }
}

/// A verified submitted form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedForm {
    context: RequestContext,
    data: FormData,
}

impl VerifiedForm {
    /// Returns the submitted data.
    pub fn data(&self) -> &FormData {
        &self.data
    }

    /// Returns the request context associated with the submission.
    pub fn context(&self) -> &RequestContext {
        &self.context
    }
}

/// A submitted form that failed request-level validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidForm {
    context: RequestContext,
    data: FormData,
    error: CsrfError,
}

impl InvalidForm {
    /// Returns the submitted data.
    pub fn data(&self) -> &FormData {
        &self.data
    }

    /// Returns the request context associated with the submission.
    pub fn context(&self) -> &RequestContext {
        &self.context
    }

    /// Returns the request-level validation error.
    pub fn error(&self) -> &CsrfError {
        &self.error
    }
}

/// Builds a submitted form from an Actix request and extractor payload.
pub fn submitted_form(
    request: &HttpRequest,
    form: web::Form<HashMap<String, String>>,
) -> FormSubmission {
    FormSubmission::from_request(request, form)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{test, web};

    use super::{FormData, FormSubmission, submitted_form};
    use crate::{CSRF_FORM_FIELD, request_context};

    #[actix_web::test]
    async fn form_data_returns_values_and_defaults() {
        let mut fields = HashMap::new();
        fields.insert("name".to_owned(), "Riccardo".to_owned());

        let data = FormData::new(fields);

        assert_eq!(data.get("name"), Some("Riccardo"));
        assert_eq!(data.value("name"), "Riccardo");
        assert_eq!(data.value("missing"), "");
        assert_eq!(data.value_or("missing", "fallback"), "fallback");
    }

    #[actix_web::test]
    async fn submitted_form_verifies_csrf_from_request_cookie() {
        let request = test::TestRequest::default().to_http_request();
        let context = request_context(&request);
        let csrf_token = context.csrf_token().to_owned();

        let mut fields = HashMap::new();
        fields.insert(CSRF_FORM_FIELD.to_owned(), csrf_token.clone());

        let verified = FormSubmission {
            context: context.clone(),
            data: FormData::new(fields),
        }
            .verify_csrf()
            .expect("csrf token should verify");

        assert_eq!(verified.data().get(CSRF_FORM_FIELD), Some(csrf_token.as_str()));
        assert_eq!(verified.context().csrf_token(), csrf_token);
    }

    #[actix_web::test]
    async fn submitted_form_preserves_invalid_payload_on_csrf_failure() {
        let request = test::TestRequest::default().to_http_request();
        let context = request_context(&request);
        let csrf_token = context.csrf_token().to_owned();

        let mut fields = HashMap::new();
        fields.insert("name".to_owned(), "Riccardo".to_owned());

        let invalid = FormSubmission {
            context: context.clone(),
            data: FormData::new(fields),
        }
            .verify_csrf()
            .expect_err("missing csrf token should fail");

        assert_eq!(invalid.data().get("name"), Some("Riccardo"));
        assert_eq!(invalid.context().csrf_token(), csrf_token);
    }

    #[actix_web::test]
    async fn submitted_form_wraps_request_context_and_data() {
        let request = test::TestRequest::default().to_http_request();

        let mut fields = HashMap::new();
        fields.insert("name".to_owned(), "Riccardo".to_owned());

        let submission = submitted_form(&request, web::Form(fields));

        assert_eq!(submission.data().get("name"), Some("Riccardo"));
        assert!(!submission.context().csrf_token().is_empty());
    }
}
