//! Actix integration for `ssw-rs`.

mod form;

use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder};
use ssw_core::{
    FlashMessage, HtmlKind, RedirectKind, Render, RequestState, Response, encode_flash_messages,
};

pub use form::{FormData, FormSubmission, InvalidForm, VerifiedForm, submitted_form};
pub use ssw_core::{CSRF_COOKIE_NAME, CSRF_FORM_FIELD, CsrfError, FLASH_COOKIE_NAME};

/// Request-scoped cookie-backed state for flash messages and CSRF tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestContext {
    state: RequestState,
}

impl RequestContext {
    /// Builds a request context from incoming cookies.
    pub fn from_request(request: &HttpRequest) -> Self {
        let flash_cookie = request.cookie(FLASH_COOKIE_NAME);
        let csrf_cookie = request.cookie(CSRF_COOKIE_NAME);

        Self {
            state: RequestState::from_cookie_values(
                flash_cookie.as_ref().map(Cookie::value),
                csrf_cookie.as_ref().map(Cookie::value),
                generate_csrf_token,
            ),
        }
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

    /// Applies pending cookie updates to a response.
    pub fn apply(&self, mut response: HttpResponse) -> HttpResponse {
        if self.state.should_clear_flash() {
            response
                .add_cookie(&removal_cookie(FLASH_COOKIE_NAME))
                .expect("failed to clear flash cookie");
        }

        if self.state.should_set_csrf_cookie() {
            response
                .add_cookie(&csrf_cookie(self.state.csrf_token()))
                .expect("failed to attach csrf cookie");
        }

        response
    }
}

/// Builds a request context from incoming Actix request cookies.
pub fn request_context(request: &HttpRequest) -> RequestContext {
    RequestContext::from_request(request)
}

/// A responder wrapper around an `ssw-core` response value.
pub struct ActixResponse(pub Response);

impl From<Response> for ActixResponse {
    fn from(response: Response) -> Self {
        Self(response)
    }
}

impl ActixResponse {
    /// Converts the wrapped response into an Actix `HttpResponse`.
    pub fn into_http_response(self) -> HttpResponse {
        to_http_response(self.0)
    }
}

impl Responder for ActixResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        self.into_http_response()
    }
}

/// Renders a document or fragment view into an Actix response.
pub fn render_html(kind: HtmlKind, view: impl Render) -> HttpResponse {
    to_http_response(Response::html_rendered(kind, view))
}

/// Renders a document or fragment view into an Actix response with an explicit status code.
pub fn render_html_with_status(status: u16, kind: HtmlKind, view: impl Render) -> HttpResponse {
    to_http_response(Response::html_rendered_with_status(status, kind, view))
}

/// Renders a full HTML document response.
pub fn page(view: impl Render) -> HttpResponse {
    render_html(HtmlKind::Document, view)
}

/// Renders a full HTML document response with an explicit status code.
pub fn page_with_status(status: u16, view: impl Render) -> HttpResponse {
    render_html_with_status(status, HtmlKind::Document, view)
}

/// Renders a full HTML document response and applies request-scoped cookies.
pub fn page_with_context(context: &RequestContext, view: impl Render) -> HttpResponse {
    context.apply(page(view))
}

/// Renders a full HTML document response with an explicit status and applies request-scoped cookies.
pub fn page_with_context_and_status(
    context: &RequestContext,
    status: u16,
    view: impl Render,
) -> HttpResponse {
    context.apply(page_with_status(status, view))
}

/// Renders a full HTML document with `422 Unprocessable Entity` and applies request-scoped cookies.
pub fn unprocessable_page(context: &RequestContext, view: impl Render) -> HttpResponse {
    page_with_context_and_status(context, 422, view)
}

/// Renders an HTML fragment response.
pub fn fragment(view: impl Render) -> HttpResponse {
    render_html(HtmlKind::Fragment, view)
}

/// Creates a `303 See Other` redirect response.
pub fn redirect(location: impl Into<String>) -> HttpResponse {
    to_http_response(Response::redirect(location))
}

/// Converts an `ssw-core` response into an Actix `HttpResponse`.
pub fn to_http_response(response: Response) -> HttpResponse {
    match response {
        Response::Html(html) => HttpResponse::build(status_code(html.status()))
            .content_type("text/html; charset=utf-8")
            .body(html.into_body()),
        Response::Text(text) => HttpResponse::build(status_code(text.status()))
            .content_type(text.content_type())
            .body(text.body().to_owned()),
        Response::Redirect(redirect) => {
            let mut response = HttpResponse::build(status_for_redirect(redirect.kind()))
                .insert_header(("Location", redirect.location().to_owned()))
                .finish();

            if !redirect.flashes().is_empty() {
                response
                    .add_cookie(&flash_cookie(redirect.flashes()))
                    .expect("failed to attach flash cookie");
            }

            response
        }
    }
}

fn status_code(status: u16) -> StatusCode {
    StatusCode::from_u16(status).expect("ssw-core responses must use valid HTTP status codes")
}

fn status_for_redirect(kind: RedirectKind) -> StatusCode {
    match kind {
        RedirectKind::SeeOther => StatusCode::SEE_OTHER,
        RedirectKind::Temporary => StatusCode::TEMPORARY_REDIRECT,
        RedirectKind::Permanent => StatusCode::PERMANENT_REDIRECT,
    }
}

fn flash_cookie(flashes: &[FlashMessage]) -> Cookie<'static> {
    Cookie::build(FLASH_COOKIE_NAME, encode_flash_messages(flashes))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish()
}

fn csrf_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(CSRF_COOKIE_NAME, token.to_owned())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish()
}

fn removal_cookie(name: &str) -> Cookie<'static> {
    let mut cookie = Cookie::build(name.to_owned(), "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();
    cookie.make_removal();
    cookie
}

fn generate_csrf_token() -> String {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).expect("OS randomness is required for CSRF tokens");
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
