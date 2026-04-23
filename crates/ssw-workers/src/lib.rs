//! Cloudflare Workers integration for `ssw-rs`.
//!
//! Runtime request and response helpers are available on `wasm32` targets only.

pub use ssw_core::{CSRF_COOKIE_NAME, CSRF_FORM_FIELD, CsrfError, FLASH_COOKIE_NAME};

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn parse_cookie_value(header: &str, name: &str) -> Option<String> {
    header.split(';').find_map(|segment| {
        let (cookie_name, value) = segment.trim().split_once('=')?;
        (cookie_name == name).then(|| value.to_owned())
    })
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn cookie_header(name: &str, value: &str) -> String {
    format!("{name}={value}; Path=/; HttpOnly; SameSite=Lax")
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn removal_cookie_header(name: &str) -> String {
    format!("{name}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax")
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use js_sys::global;
    use wasm_bindgen::JsCast;
    use web_sys::WorkerGlobalScope;
    use worker::{
        FormData as WorkerFormData, Request, Response as WorkerResponse, Result as WorkerResult,
    };

    use ssw_core::{
        FlashMessage, HtmlKind, RedirectKind, Render, RequestState, Response, encode_flash_messages,
    };

    use super::{
        CSRF_COOKIE_NAME, CSRF_FORM_FIELD, CsrfError, FLASH_COOKIE_NAME, cookie_header,
        parse_cookie_value, removal_cookie_header,
    };

    /// Request-scoped cookie-backed state for flash messages and CSRF tokens.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RequestContext {
        state: RequestState,
    }

    impl RequestContext {
        /// Builds a request context from incoming Worker request headers.
        pub fn from_request(request: &Request) -> WorkerResult<Self> {
            let flash_cookie = cookie_value(request, FLASH_COOKIE_NAME)?;
            let csrf_cookie = cookie_value(request, CSRF_COOKIE_NAME)?;

            Ok(Self {
                state: RequestState::from_cookie_values(
                    flash_cookie.as_deref(),
                    csrf_cookie.as_deref(),
                    generate_csrf_token,
                ),
            })
        }

        /// Returns the flash messages attached to the current request.
        pub fn flashes(&self) -> &[FlashMessage] {
            self.state.flashes()
        }

        /// Returns the CSRF token for the current request.
        pub fn csrf_token(&self) -> &str {
            self.state.csrf_token()
        }

        /// Verifies a submitted form token against the request token.
        pub fn verify_csrf(&self, form_token: Option<&str>) -> Result<(), CsrfError> {
            self.state.verify_csrf(form_token)
        }

        /// Applies pending cookie updates to a Worker response.
        pub fn apply(&self, mut response: WorkerResponse) -> WorkerResult<WorkerResponse> {
            if self.state.should_clear_flash() {
                response
                    .headers_mut()
                    .append("Set-Cookie", &removal_cookie_header(FLASH_COOKIE_NAME))?;
            }

            if self.state.should_set_csrf_cookie() {
                response.headers_mut().append(
                    "Set-Cookie",
                    &cookie_header(CSRF_COOKIE_NAME, self.state.csrf_token()),
                )?;
            }

            Ok(response)
        }
    }

    /// Builds a request context from incoming Worker request headers.
    pub fn request_context(request: &Request) -> WorkerResult<RequestContext> {
        RequestContext::from_request(request)
    }

    /// Renders a document or fragment view into a Worker response.
    pub fn render_html(kind: HtmlKind, view: impl Render) -> WorkerResult<WorkerResponse> {
        to_worker_response(Response::html_rendered(kind, view))
    }

    /// Renders a document or fragment view into a Worker response with an explicit status code.
    pub fn render_html_with_status(
        status: u16,
        kind: HtmlKind,
        view: impl Render,
    ) -> WorkerResult<WorkerResponse> {
        to_worker_response(Response::html_rendered_with_status(status, kind, view))
    }

    /// Renders a full HTML document response.
    pub fn page(view: impl Render) -> WorkerResult<WorkerResponse> {
        render_html(HtmlKind::Document, view)
    }

    /// Renders a full HTML document response with an explicit status code.
    pub fn page_with_status(status: u16, view: impl Render) -> WorkerResult<WorkerResponse> {
        render_html_with_status(status, HtmlKind::Document, view)
    }

    /// Renders a full HTML document response and applies request-scoped cookies.
    pub fn page_with_context(
        context: &RequestContext,
        view: impl Render,
    ) -> WorkerResult<WorkerResponse> {
        context.apply(page(view)?)
    }

    /// Renders a full HTML document response with an explicit status and applies request-scoped cookies.
    pub fn page_with_context_and_status(
        context: &RequestContext,
        status: u16,
        view: impl Render,
    ) -> WorkerResult<WorkerResponse> {
        context.apply(page_with_status(status, view)?)
    }

    /// Renders a full HTML document with `422 Unprocessable Entity` and applies request-scoped cookies.
    pub fn unprocessable_page(
        context: &RequestContext,
        view: impl Render,
    ) -> WorkerResult<WorkerResponse> {
        page_with_context_and_status(context, 422, view)
    }

    /// Renders an HTML fragment response.
    pub fn fragment(view: impl Render) -> WorkerResult<WorkerResponse> {
        render_html(HtmlKind::Fragment, view)
    }

    /// Request form data with small helpers for common server-side access patterns.
    pub struct FormData {
        inner: WorkerFormData,
    }

    impl FormData {
        /// Builds form data from a Worker `FormData` value.
        pub fn new(inner: WorkerFormData) -> Self {
            Self { inner }
        }

        /// Extracts submitted form data from a Worker request.
        pub async fn from_request(request: &mut Request) -> WorkerResult<Self> {
            Ok(Self::new(request.form_data().await?))
        }

        /// Returns the submitted value for a text field.
        pub fn get(&self, name: &str) -> Option<String> {
            self.inner.get_field(name)
        }

        /// Returns a field value or the empty string when the field is missing.
        pub fn value(&self, name: &str) -> String {
            self.get(name).unwrap_or_default()
        }

        /// Returns a field value or a provided default when the field is missing.
        pub fn value_or(&self, name: &str, default: &str) -> String {
            self.get(name).unwrap_or_else(|| default.to_owned())
        }
    }

    /// A submitted Worker form paired with the current request context.
    pub struct FormSubmission {
        context: RequestContext,
        data: FormData,
    }

    impl FormSubmission {
        /// Builds a submitted form from a Worker request.
        pub async fn from_request(request: &mut Request) -> WorkerResult<Self> {
            Ok(Self {
                context: request_context(request)?,
                data: FormData::from_request(request).await?,
            })
        }

        /// Returns the submitted data.
        pub fn data(&self) -> &FormData {
            &self.data
        }

        /// Returns the request context associated with the submission.
        pub fn context(&self) -> &RequestContext {
            &self.context
        }

        /// Verifies the submitted CSRF token and returns a verified form on success.
        pub fn verify_csrf(self) -> Result<VerifiedForm, InvalidForm> {
            match self
                .context
                .verify_csrf(self.data.get(CSRF_FORM_FIELD).as_deref())
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

    /// A verified submitted Worker form.
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

    /// A submitted Worker form that failed request-level validation.
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

    /// Builds a submitted form from a Worker request.
    pub async fn submitted_form(request: &mut Request) -> WorkerResult<FormSubmission> {
        FormSubmission::from_request(request).await
    }

    /// Creates a `303 See Other` redirect response.
    pub fn redirect(location: impl Into<String>) -> WorkerResult<WorkerResponse> {
        to_worker_response(Response::redirect(location))
    }

    /// Converts an `ssw-core` response into a Workers response.
    pub fn to_worker_response(response: Response) -> WorkerResult<WorkerResponse> {
        match response {
            Response::Html(html) => {
                let mut response = WorkerResponse::from_html(html.body())?;
                response = response.with_status(html.status());
                Ok(response)
            }
            Response::Text(text) => {
                let mut response = WorkerResponse::ok(text.body().to_owned())?;
                response = response.with_status(text.status());
                response
                    .headers_mut()
                    .set("Content-Type", text.content_type())?;
                Ok(response)
            }
            Response::Redirect(redirect) => redirect_response(redirect),
        }
    }

    fn redirect_response(redirect: ssw_core::Redirect) -> WorkerResult<WorkerResponse> {
        let mut response =
            WorkerResponse::empty()?.with_status(status_for_redirect(redirect.kind()));
        response
            .headers_mut()
            .set("Location", redirect.location())?;

        if !redirect.flashes().is_empty() {
            response.headers_mut().append(
                "Set-Cookie",
                &cookie_header(
                    FLASH_COOKIE_NAME,
                    &encode_flash_messages(redirect.flashes()),
                ),
            )?;
        }

        Ok(response)
    }

    fn cookie_value(request: &Request, name: &str) -> WorkerResult<Option<String>> {
        Ok(request
            .headers()
            .get("Cookie")?
            .and_then(|header| parse_cookie_value(&header, name)))
    }

    fn generate_csrf_token() -> String {
        let global: WorkerGlobalScope = global().unchecked_into();
        let crypto = global
            .crypto()
            .expect("Workers Web Crypto API is required for CSRF tokens");

        let mut bytes = [0_u8; 32];
        crypto
            .get_random_values_with_u8_array(&mut bytes)
            .expect("Workers Web Crypto API is required for CSRF tokens");

        encode_hex(&bytes)
    }

    fn encode_hex(bytes: &[u8]) -> String {
        let mut output = String::with_capacity(bytes.len() * 2);

        for byte in bytes {
            output.push(char::from(b"0123456789abcdef"[(byte >> 4) as usize]));
            output.push(char::from(b"0123456789abcdef"[(byte & 0x0f) as usize]));
        }

        output
    }

    fn status_for_redirect(kind: RedirectKind) -> u16 {
        match kind {
            RedirectKind::SeeOther => 303,
            RedirectKind::Temporary => 307,
            RedirectKind::Permanent => 308,
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(test)]
mod tests {
    use super::{cookie_header, parse_cookie_value, removal_cookie_header};

    #[test]
    fn parses_cookie_values_from_header() {
        let header = "theme=light; ssw-flash=info~6869; ssw-csrf=deadbeef";

        assert_eq!(
            parse_cookie_value(header, "ssw-flash"),
            Some("info~6869".to_owned())
        );
        assert_eq!(
            parse_cookie_value(header, "ssw-csrf"),
            Some("deadbeef".to_owned())
        );
        assert_eq!(parse_cookie_value(header, "missing"), None);
    }

    #[test]
    fn builds_cookie_headers_with_expected_attributes() {
        assert_eq!(
            cookie_header("ssw-csrf", "abc123"),
            "ssw-csrf=abc123; Path=/; HttpOnly; SameSite=Lax"
        );
        assert_eq!(
            removal_cookie_header("ssw-flash"),
            "ssw-flash=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"
        );
    }
}
