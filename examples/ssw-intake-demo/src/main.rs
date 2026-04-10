use std::collections::HashMap;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use ssw_actix::{CSRF_FORM_FIELD, page_with_context, request_context, to_http_response};
use ssw_components::{
    ButtonVariant, Field, SelectOption, alert, button, button_with_variant, container, email_input,
    flash_notice, hidden_input, section, select, stack, submit_button, text_input, textarea,
};
use ssw_core::{FlashMessage, HtmlKind, Response};
use ssw_html::{Markup, html, page as html_page};

const COMPONENT_CSS: &str = include_str!("../../../styles/ssw-components-default.css");
const APP_CSS: &str = r#"
body {
  margin: 0;
  background:
    radial-gradient(circle at top, rgb(21 101 192 / 0.08), transparent 28rem),
    linear-gradient(180deg, #f7fafc 0%, #edf2f7 100%);
  color: #1f2933;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
}

a {
  color: inherit;
}

.demo-link {
  color: #1565c0;
  font-weight: 600;
  text-decoration: none;
}

.demo-link:hover {
  text-decoration: underline;
}

.demo-shell {
  display: grid;
  gap: 1.5rem;
  padding: 3rem 0 4rem;
}

.demo-hero {
  display: grid;
  gap: 1rem;
}

.demo-kicker {
  margin: 0;
  color: #1565c0;
  font-size: 0.85rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.demo-title {
  margin: 0;
  font-size: clamp(2rem, 5vw, 3.75rem);
  line-height: 0.95;
}

.demo-copy {
  max-width: 42rem;
  margin: 0;
  color: #52606d;
  font-size: 1.05rem;
  line-height: 1.6;
}

.demo-grid {
  display: grid;
  gap: 1.5rem;
}

.demo-points {
  margin: 0;
  padding-left: 1.1rem;
  color: #52606d;
  line-height: 1.6;
}

.demo-form {
  display: grid;
  gap: 1rem;
}

.demo-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.demo-style-grid {
  display: grid;
  gap: 1rem;
}

.demo-inline {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: center;
}

.demo-card-title {
  margin: 0;
  font-size: 1.25rem;
}

.demo-card-copy {
  margin: 0;
  color: #52606d;
  line-height: 1.6;
}

@media (min-width: 56rem) {
  .demo-grid {
    grid-template-columns: 1.1fr 0.9fr;
    align-items: start;
  }
}
"#;

#[derive(Debug, Clone, Default)]
struct IntakeField {
    value: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct IntakeFormState {
    name: IntakeField,
    email: IntakeField,
    track: IntakeField,
    message: IntakeField,
    summary_error: Option<String>,
}

impl IntakeFormState {
    fn has_errors(&self) -> bool {
        self.summary_error.is_some()
    }
}

fn intake_state_from(form: &HashMap<String, String>) -> IntakeFormState {
    IntakeFormState {
        name: IntakeField {
            value: form.get("name").cloned().unwrap_or_default(),
            error: None,
        },
        email: IntakeField {
            value: form.get("email").cloned().unwrap_or_default(),
            error: None,
        },
        track: IntakeField {
            value: form.get("track").cloned().unwrap_or_default(),
            error: None,
        },
        message: IntakeField {
            value: form.get("message").cloned().unwrap_or_default(),
            error: None,
        },
        summary_error: None,
    }
}

fn validate_intake(form: &HashMap<String, String>) -> IntakeFormState {
    let mut state = intake_state_from(form);

    if state.name.value.trim().is_empty() {
        state.name.error = Some("Name is required.".to_owned());
    }

    if state.email.value.trim().is_empty() {
        state.email.error = Some("Email is required.".to_owned());
    } else if !state.email.value.contains('@') {
        state.email.error = Some("Email must look valid.".to_owned());
    }

    if state.track.value.trim().is_empty() {
        state.track.error = Some("Pick a project track.".to_owned());
    }

    if state.message.value.trim().len() < 12 {
        state.message.error = Some("Message should give a little more context.".to_owned());
    }

    if state.name.error.is_some()
        || state.email.error.is_some()
        || state.track.error.is_some()
        || state.message.error.is_some()
    {
        state.summary_error = Some("Please fix the highlighted fields.".to_owned());
    }

    state
}

fn track_options() -> [SelectOption<'static>; 4] {
    [
        SelectOption::new("", "Choose a track"),
        SelectOption::new("launch", "Launch sprint"),
        SelectOption::new("migration", "Migration"),
        SelectOption::new("audit", "Architecture audit"),
    ]
}

fn app_page(title: &str, content: Markup) -> Markup {
    html_page(title)
        .head(html! {
            link rel="stylesheet" href="/app.css";
        })
        .body(html! {
            (container(html! {
                div class="demo-shell" {
                    (content)
                }
            }))
        })
        .render()
}

fn intake_page(state: &IntakeFormState, flashes: &[FlashMessage], csrf_token: &str) -> Markup {
    let name = Field::new("name", "name", "Name")
        .value(state.name.value.as_str())
        .error(state.name.error.as_deref())
        .required(true);
    let email = Field::new("email", "email", "Email")
        .value(state.email.value.as_str())
        .error(state.email.error.as_deref())
        .required(true);
    let track = Field::new("track", "track", "Project track")
        .value(state.track.value.as_str())
        .error(state.track.error.as_deref())
        .required(true);
    let message = Field::new("message", "message", "Project brief")
        .value(state.message.value.as_str())
        .error(state.message.error.as_deref())
        .required(true);
    let track_options = track_options();

    app_page(
        "ssw-rs Intake Demo",
        html! {
            div class="demo-hero" {
                p class="demo-kicker" { "Server Side Web" }
                h1 class="demo-title" { "A small intake flow, rendered on the server." }
                p class="demo-copy" {
                    "This example uses the current ssw-rs stack: document rendering, stable component classes, form fields, select, flash messages, CSRF protection, and a linked first-party stylesheet."
                }
                p class="demo-copy" {
                    a class="demo-link" href="/style-guide" { "Browse the live style guide" }
                }
            }

            div class="demo-grid" {
                (section(stack(html! {
                    h2 class="demo-card-title" { "Why this example exists" }
                    p class="demo-card-copy" {
                        "It is intentionally narrow. The goal is to pressure the current primitives in a real route flow before the framework grows more abstraction."
                    }
                    ul class="demo-points" {
                        li { "Layout wrappers and section surfaces" }
                        li { "Field, input, textarea, and select helpers" }
                        li { "Flash messages across redirects" }
                        li { "Cookie-backed CSRF hooks" }
                    }
                    p class="demo-card-copy" {
                        "Use the style guide route to inspect the current primitives outside the intake flow."
                    }
                })))

                (section(html! {
                    div class="ssw-stack" {
                        @for flash in flashes {
                            (flash_notice(flash))
                        }

                        @if state.summary_error.is_some() {
                            (flash_notice(&FlashMessage::error(
                                state.summary_error.as_deref().unwrap(),
                            )))
                        }

                        h2 class="demo-card-title" { "Start a project" }
                        p class="demo-card-copy" {
                            "Send a short intake note. Successful submissions redirect with a flash notice; invalid ones stay on the same page with preserved values."
                        }

                        form class="demo-form" method="post" action="/intake" {
                            (hidden_input(CSRF_FORM_FIELD, csrf_token))
                            (text_input(&name))
                            (email_input(&email))
                            (select(&track, &track_options))
                            (textarea(&message, 5))
                            div class="demo-actions" {
                                (submit_button("Send request"))
                            }
                        }
                    }
                }))
            }
        },
    )
}

fn style_guide_page() -> Markup {
    let valid_name = Field::new("preview-name", "preview_name", "Preview field")
        .value("Riccardo")
        .required(true);
    let invalid_track = Field::new("preview-track", "preview_track", "Invalid select")
        .value("")
        .error(Some("A selection is required."))
        .required(true);
    let preview_message = Field::new("preview-message", "preview_message", "Textarea")
        .value("Server-rendered interfaces can still feel polished.")
        .required(true);
    let options = track_options();

    app_page(
        "Component style guide",
        html! {
            div class="demo-hero" {
                p class="demo-kicker" { "Component Preview" }
                h1 class="demo-title" { "A live style guide for the current primitives." }
                p class="demo-copy" {
                    "This page exists to make visual review cheap. It is not a design system yet, but it gives us a real place to inspect structure, spacing, and state styling."
                }
                p class="demo-copy" {
                    a class="demo-link" href="/" { "Back to the intake demo" }
                }
            }

            div class="demo-grid" {
                (section(stack(html! {
                    h2 class="demo-card-title" { "Notices and actions" }
                    p class="demo-card-copy" { "These are the current feedback and action primitives." }
                    div class="demo-style-grid" {
                        (alert("Informational notice"))
                        (flash_notice(&FlashMessage::success("Successful flash message")))
                        (flash_notice(&FlashMessage::error("Error flash message")))
                    }
                    div class="demo-inline" {
                        (button("Primary button"))
                        (button_with_variant("Secondary button", ButtonVariant::Secondary))
                    }
                })))

                (section(stack(html! {
                    h2 class="demo-card-title" { "Fields and states" }
                    p class="demo-card-copy" { "Inputs, textarea, and select should remain legible without JavaScript." }
                    (text_input(&valid_name))
                    (select(&invalid_track, &options))
                    (textarea(&preview_message, 4))
                })))
            }
        },
    )
}

fn thanks_page(flashes: &[FlashMessage]) -> Markup {
    app_page(
        "Request sent",
        html! {
            (section(stack(html! {
                @for flash in flashes {
                    (flash_notice(flash))
                }
                h1 class="demo-card-title" { "Request sent" }
                p class="demo-card-copy" {
                    "The redirect, flash message, and success page are all coming from the current ssw-rs request model."
                }
                p class="demo-card-copy" {
                    a href="/" { "Back to the intake form" }
                }
            })))
        },
    )
}

async fn stylesheet() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(format!("{COMPONENT_CSS}\n{APP_CSS}"))
}

async fn intake_get(request: HttpRequest) -> HttpResponse {
    let context = request_context(&request);

    page_with_context(
        &context,
        intake_page(
            &IntakeFormState::default(),
            context.flashes(),
            context.csrf_token(),
        ),
    )
}

async fn intake_post(
    request: HttpRequest,
    form: web::Form<HashMap<String, String>>,
) -> HttpResponse {
    let context = request_context(&request);

    if context
        .verify_csrf(form.get(CSRF_FORM_FIELD).map(String::as_str))
        .is_err()
    {
        let mut state = intake_state_from(&form);
        state.summary_error = Some("Your form expired. Reload the page and try again.".to_owned());

        return page_with_context(
            &context,
            intake_page(&state, context.flashes(), context.csrf_token()),
        );
    }

    let state = validate_intake(&form);
    if state.has_errors() {
        return page_with_context(
            &context,
            intake_page(&state, context.flashes(), context.csrf_token()),
        );
    }

    to_http_response(Response::redirect_with_flash(
        "/thanks",
        FlashMessage::success("Request queued. We will reply with a scope outline shortly."),
    ))
}

async fn thanks(request: HttpRequest) -> HttpResponse {
    let context = request_context(&request);

    page_with_context(&context, thanks_page(context.flashes()))
}

async fn style_guide() -> HttpResponse {
    to_http_response(Response::html_rendered(
        HtmlKind::Document,
        style_guide_page(),
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let address = format!("127.0.0.1:{port}");

    println!("ssw-intake-demo listening on http://{address}");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(intake_get))
            .route("/intake", web::post().to(intake_post))
            .route("/thanks", web::get().to(thanks))
            .route("/style-guide", web::get().to(style_guide))
            .route("/app.css", web::get().to(stylesheet))
    })
    .bind(address)?
    .run()
    .await
}
