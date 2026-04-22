//! Actix integration for `ssw-rs`.

use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder};
use ssw_core::{
    FlashMessage, HtmlKind, RedirectKind, Render, RequestState, Response, encode_flash_messages,
};

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::body::to_bytes;
    use actix_web::cookie::Cookie;
    use actix_web::http::{StatusCode, header};
    use actix_web::test;
    use actix_web::{App, HttpRequest, HttpResponse, web};
    use ssw_components::{Field, email_input, flash_notice, hidden_input, text_input, textarea};
    use ssw_core::{FlashMessage, HtmlKind, Response};
    use ssw_html::{Markup, html, page as html_page};

    use super::{
        ActixResponse, CSRF_COOKIE_NAME, CSRF_FORM_FIELD, FLASH_COOKIE_NAME, fragment, page,
        page_with_context, page_with_context_and_status, page_with_status, redirect, render_html,
        render_html_with_status, request_context, to_http_response,
    };

    #[actix_web::test]
    async fn converts_html_response() {
        let response = to_http_response(Response::html(HtmlKind::Document, "<h1>home</h1>"));

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );

        let body = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body, "<h1>home</h1>");
    }

    #[actix_web::test]
    async fn preserves_non_200_statuses_from_core_response() {
        let response = to_http_response(Response::html_with_status(
            404,
            HtmlKind::Document,
            "<h1>missing</h1>",
        ));

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body, "<h1>missing</h1>");
    }

    #[actix_web::test]
    async fn renders_from_core_trait() {
        let response = render_html(HtmlKind::Fragment, Markup::text("Hello from Actix"));
        let body = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body, "Hello from Actix");
    }

    #[actix_web::test]
    async fn renders_html_with_status_helpers() {
        let fragment_response =
            render_html_with_status(422, HtmlKind::Fragment, Markup::text("Problem"));
        assert_eq!(fragment_response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let fragment_body = to_bytes(fragment_response.into_body()).await.unwrap();
        assert_eq!(fragment_body, "Problem");

        let page_response = page_with_status(404, Markup::text("<h1>missing</h1>"));
        assert_eq!(page_response.status(), StatusCode::NOT_FOUND);
    }

    fn app_layout(title: &str, content: Markup) -> Markup {
        html_page(title)
            .body_class("app")
            .head(html! {
                link rel="stylesheet" href="/app.css";
            })
            .body(html! {
                main #app .shell {
                    header .topbar {
                        h1 { (title) }
                    }
                    (content)
                }
            })
            .render()
    }

    #[derive(Debug, Clone, Default)]
    struct ContactField {
        value: String,
        error: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    struct ContactFormState {
        name: ContactField,
        email: ContactField,
        message: ContactField,
        summary_error: Option<String>,
    }

    impl ContactFormState {
        fn has_errors(&self) -> bool {
            self.summary_error.is_some()
        }
    }

    fn validate_contact_form(form: &HashMap<String, String>) -> ContactFormState {
        let mut state = contact_form_state_from(form);

        if state.name.value.trim().is_empty() {
            state.name.error = Some("Name is required.".to_owned());
        }

        if state.email.value.trim().is_empty() {
            state.email.error = Some("Email is required.".to_owned());
        } else if !state.email.value.contains('@') {
            state.email.error = Some("Email must look valid.".to_owned());
        }

        if state.name.error.is_some() || state.email.error.is_some() {
            state.summary_error = Some("Please fix the highlighted fields.".to_owned());
        }

        state
    }

    fn contact_form_state_from(form: &HashMap<String, String>) -> ContactFormState {
        ContactFormState {
            name: ContactField {
                value: form.get("name").cloned().unwrap_or_default(),
                error: None,
            },
            email: ContactField {
                value: form.get("email").cloned().unwrap_or_default(),
                error: None,
            },
            message: ContactField {
                value: form.get("message").cloned().unwrap_or_default(),
                error: None,
            },
            summary_error: None,
        }
    }

    fn contact_page(
        state: &ContactFormState,
        flashes: &[FlashMessage],
        csrf_token: &str,
    ) -> Markup {
        let name = Field::new("name", "name", "Name")
            .value(state.name.value.as_str())
            .error(state.name.error.as_deref())
            .required(true);
        let email = Field::new("email", "email", "Email")
            .value(state.email.value.as_str())
            .error(state.email.error.as_deref())
            .required(true);
        let message = Field::new("message", "message", "Message")
            .value(state.message.value.as_str())
            .error(state.message.error.as_deref());

        app_layout(
            "Contact",
            html! {
                section .panel {
                    h2 { "Contact us" }
                    p { "Send a simple server-rendered form." }

                    @for flash in flashes {
                        (flash_notice(flash))
                    }

                    @if state.summary_error.is_some() {
                        div .notice .notice_error role="alert" {
                            p { (state.summary_error.as_deref().unwrap()) }
                            ul {
                                @if state.name.error.is_some() {
                                    li { (state.name.error.as_deref().unwrap()) }
                                }
                                @if state.email.error.is_some() {
                                    li { (state.email.error.as_deref().unwrap()) }
                                }
                            }
                        }
                    }

                    form method="post" action="/contact" {
                        (hidden_input(CSRF_FORM_FIELD, csrf_token))
                        (text_input(&name))
                        (email_input(&email))
                        (textarea(&message, 4))
                        button type="submit" { "Send" }
                    }
                }
            },
        )
    }

    fn thanks_page(flashes: &[FlashMessage]) -> Markup {
        app_layout(
            "Thanks",
            html! {
                section .panel .panel_success {
                    @for flash in flashes {
                        (flash_notice(flash))
                    }

                    h2 { "Message sent" }
                    p { "Your form was handled on the server and redirected cleanly." }
                }
            },
        )
    }

    async fn home() -> HttpResponse {
        page(app_layout(
            "Dashboard",
            html! {
                section .panel {
                    p { "Rendered on the server." }
                    a href="/panel" { "Load panel" }
                }
            },
        ))
    }

    async fn panel() -> HttpResponse {
        fragment(html! {
            section .panel .panel_alt {
                h2 { "Panel" }
                p { "This fragment can be swapped into the page." }
            }
        })
    }

    async fn submit() -> ActixResponse {
        Response::redirect("/").into()
    }

    async fn legacy_redirect() -> HttpResponse {
        redirect("/panel")
    }

    async fn contact_get(request: HttpRequest) -> HttpResponse {
        let context = request_context(&request);

        page_with_context(
            &context,
            contact_page(
                &ContactFormState::default(),
                context.flashes(),
                context.csrf_token(),
            ),
        )
    }

    async fn contact_post(
        request: HttpRequest,
        form: web::Form<HashMap<String, String>>,
    ) -> HttpResponse {
        let context = request_context(&request);

        if context
            .verify_csrf(form.get(CSRF_FORM_FIELD).map(String::as_str))
            .is_err()
        {
            let mut state = contact_form_state_from(&form);
            state.summary_error =
                Some("Your form expired. Reload the page and try again.".to_owned());

            return page_with_context_and_status(
                &context,
                422,
                contact_page(&state, context.flashes(), context.csrf_token()),
            );
        }

        let state = validate_contact_form(&form);

        if state.has_errors() {
            return page_with_context_and_status(
                &context,
                422,
                contact_page(&state, context.flashes(), context.csrf_token()),
            );
        }

        to_http_response(Response::redirect_with_flash(
            "/thanks",
            FlashMessage::success("Your message was sent."),
        ))
    }

    async fn thanks(request: HttpRequest) -> HttpResponse {
        let context = request_context(&request);

        page_with_context(&context, thanks_page(context.flashes()))
    }

    fn response_cookie(headers: &header::HeaderMap, name: &str) -> Option<Cookie<'static>> {
        headers
            .get_all(header::SET_COOKIE)
            .filter_map(|value| value.to_str().ok())
            .filter_map(|value| Cookie::parse(value.to_owned()).ok())
            .map(Cookie::into_owned)
            .find(|cookie| cookie.name() == name)
    }

    #[actix_web::test]
    async fn serves_page_fragment_and_redirect_flow() {
        let app = test::init_service(
            App::new()
                .route("/", web::get().to(home))
                .route("/panel", web::get().to(panel))
                .route("/submit", web::post().to(submit))
                .route("/legacy", web::post().to(legacy_redirect)),
        )
        .await;

        let home_response =
            test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
        assert_eq!(home_response.status(), StatusCode::OK);
        assert_eq!(
            home_response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        let home_body = to_bytes(home_response.into_body()).await.unwrap();
        let home_body = std::str::from_utf8(&home_body).unwrap();
        assert!(home_body.starts_with("<!DOCTYPE html>"));
        assert!(home_body.contains("<body class=\"app\">"));
        assert!(home_body.contains("<main id=\"app\" class=\"shell\">"));
        assert!(home_body.contains("<a href=\"/panel\">Load panel</a>"));

        let panel_response =
            test::call_service(&app, test::TestRequest::get().uri("/panel").to_request()).await;
        assert_eq!(panel_response.status(), StatusCode::OK);
        let panel_body = to_bytes(panel_response.into_body()).await.unwrap();
        assert_eq!(
            std::str::from_utf8(&panel_body).unwrap(),
            "<section class=\"panel panel-alt\"><h2>Panel</h2><p>This fragment can be swapped into the page.</p></section>"
        );

        let submit_response =
            test::call_service(&app, test::TestRequest::post().uri("/submit").to_request()).await;
        assert_eq!(submit_response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            submit_response.headers().get(header::LOCATION).unwrap(),
            "/"
        );

        let legacy_response =
            test::call_service(&app, test::TestRequest::post().uri("/legacy").to_request()).await;
        assert_eq!(legacy_response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            legacy_response.headers().get(header::LOCATION).unwrap(),
            "/panel"
        );
    }

    #[actix_web::test]
    async fn handles_form_validation_and_success_redirect() {
        let app = test::init_service(
            App::new()
                .route("/contact", web::get().to(contact_get))
                .route("/contact", web::post().to(contact_post))
                .route("/thanks", web::get().to(thanks)),
        )
        .await;

        let contact_response =
            test::call_service(&app, test::TestRequest::get().uri("/contact").to_request()).await;
        assert_eq!(contact_response.status(), StatusCode::OK);
        let csrf_cookie = response_cookie(contact_response.headers(), CSRF_COOKIE_NAME).unwrap();
        let csrf_token = csrf_cookie.value().to_owned();
        let contact_body = to_bytes(contact_response.into_body()).await.unwrap();
        let contact_body = std::str::from_utf8(&contact_body).unwrap();
        assert!(contact_body.contains("<form method=\"post\" action=\"/contact\">"));
        assert!(contact_body.contains("type=\"hidden\" name=\"csrf_token\""));
        assert!(contact_body.contains(&format!("value=\"{csrf_token}\"")));
        assert!(contact_body.contains("class=\"ssw-input\""));
        assert!(
            contact_body.contains("id=\"name\" type=\"text\" name=\"name\" value=\"\" required")
        );
        assert!(
            contact_body.contains("id=\"email\" type=\"email\" name=\"email\" value=\"\" required")
        );

        let csrf_error_response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/contact")
                .cookie(Cookie::new(CSRF_COOKIE_NAME, csrf_token.clone()))
                .set_form([
                    (CSRF_FORM_FIELD, "wrong-token"),
                    ("name", "Riccardo"),
                    ("email", "sprite@example.com"),
                    ("message", "Hello"),
                ])
                .to_request(),
        )
        .await;
        assert_eq!(
            csrf_error_response.status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        let csrf_error_body = to_bytes(csrf_error_response.into_body()).await.unwrap();
        let csrf_error_body = std::str::from_utf8(&csrf_error_body).unwrap();
        assert!(csrf_error_body.contains("Your form expired. Reload the page and try again."));
        assert!(csrf_error_body.contains("value=\"Riccardo\""));
        assert!(csrf_error_body.contains("value=\"sprite@example.com\""));

        let invalid_response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/contact")
                .cookie(Cookie::new(CSRF_COOKIE_NAME, csrf_token.clone()))
                .set_form([
                    (CSRF_FORM_FIELD, csrf_token.as_str()),
                    ("name", ""),
                    ("email", "sprite-at-example.com"),
                    ("message", "Hello"),
                ])
                .to_request(),
        )
        .await;
        assert_eq!(invalid_response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let invalid_body = to_bytes(invalid_response.into_body()).await.unwrap();
        let invalid_body = std::str::from_utf8(&invalid_body).unwrap();
        assert!(invalid_body.contains("Please fix the highlighted fields."));
        assert!(invalid_body.contains("Name is required."));
        assert!(invalid_body.contains("Email must look valid."));
        assert!(invalid_body.contains("value=\"sprite-at-example.com\""));
        assert!(invalid_body.contains("aria-invalid=\"true\""));
        assert!(invalid_body.contains("data-invalid=\"true\""));
        assert!(invalid_body.contains("aria-describedby=\"name-error\""));
        assert!(invalid_body.contains("aria-describedby=\"email-error\""));
        assert!(
            invalid_body
                .contains("<textarea class=\"ssw-textarea\" id=\"message\" name=\"message\" rows=\"4\">Hello</textarea>")
        );

        let valid_response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/contact")
                .cookie(Cookie::new(CSRF_COOKIE_NAME, csrf_token.clone()))
                .set_form([
                    (CSRF_FORM_FIELD, csrf_token.as_str()),
                    ("name", "Riccardo"),
                    ("email", "sprite@example.com"),
                    ("message", "Shipping a server-first app"),
                ])
                .to_request(),
        )
        .await;
        assert_eq!(valid_response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            valid_response.headers().get(header::LOCATION).unwrap(),
            "/thanks"
        );
        let flash_cookie = response_cookie(valid_response.headers(), FLASH_COOKIE_NAME).unwrap();

        let thanks_response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/thanks")
                .cookie(Cookie::new(CSRF_COOKIE_NAME, csrf_token.clone()))
                .cookie(Cookie::new(
                    FLASH_COOKIE_NAME,
                    flash_cookie.value().to_owned(),
                ))
                .to_request(),
        )
        .await;
        assert_eq!(thanks_response.status(), StatusCode::OK);
        let cleared_flash_cookie =
            response_cookie(thanks_response.headers(), FLASH_COOKIE_NAME).unwrap();
        assert_eq!(cleared_flash_cookie.value(), "");
        let thanks_body = to_bytes(thanks_response.into_body()).await.unwrap();
        let thanks_body = std::str::from_utf8(&thanks_body).unwrap();
        assert!(thanks_body.contains("Message sent"));
        assert!(thanks_body.contains("handled on the server"));
        assert!(thanks_body.contains("Your message was sent."));

        let second_thanks_response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/thanks")
                .cookie(Cookie::new(CSRF_COOKIE_NAME, csrf_token))
                .to_request(),
        )
        .await;
        let second_thanks_body = to_bytes(second_thanks_response.into_body()).await.unwrap();
        let second_thanks_body = std::str::from_utf8(&second_thanks_body).unwrap();
        assert!(!second_thanks_body.contains("Your message was sent."));
    }
}
