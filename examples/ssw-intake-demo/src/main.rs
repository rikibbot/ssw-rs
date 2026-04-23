//! Example Actix app for the narrow intake-flow slice.
//!
//! This binary exists to pressure:
//! - form helpers and invalid redisplay
//! - flash and CSRF hooks
//! - page-shell and field primitives
//! - the `/style-guide` visual review route

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use ssw_actix::{
    CSRF_FORM_FIELD, FormData, page_with_context, request_context, submitted_form,
    to_http_response, unprocessable_page,
};
use ssw_components::{
    BadgeVariant, BreadcrumbItem, ButtonVariant, Field, PaginationItem, SelectOption, StatItem,
    TableCell, TableRow, ValidationItem, alert, badge, badge_with_variant, breadcrumbs, button,
    button_with_variant, card_header, container, data_table, email_input, flash_notice,
    hidden_input, link_button, page_actions, page_header, page_shell, pagination, section, select,
    stack, stat_list, submit_button, text_input, textarea, validation_summary,
};
use ssw_core::{FlashMessage, HtmlKind, Response};
use ssw_css::css;
use ssw_html::{Markup, assets, fonts, html, page as html_page};

const THEME_CSS: &str = include_str!("../../../styles/ssw-theme-default.css");
const APP_STYLESHEET_PATH: &str = "/assets/app.css";
const APP_CSS: &str = r#"
body {
  margin: 0;
  background: linear-gradient(180deg, #fcfcfc 0%, #f4f4f5 100%);
  color: var(--ssw-color-text, #09090b);
  font-family: var(--ssw-font-body, "Inter", "Segoe UI", sans-serif);
}

a {
  color: inherit;
}

.demo-grid {
  display: grid;
  gap: 1rem;
  align-items: start;
}

.demo-points {
  margin: 0;
  padding-left: 1.1rem;
  color: #52525b;
  line-height: 1.6;
}

.demo-form {
  display: grid;
  gap: 0.9rem;
}

.demo-style-grid {
  display: grid;
  gap: 0.75rem;
}

.demo-inline {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  align-items: center;
}

@media (min-width: 56rem) {
  .demo-grid {
    grid-template-columns: minmax(0, 1.05fr) minmax(0, 0.95fr);
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

fn intake_state_from(form: &FormData) -> IntakeFormState {
    IntakeFormState {
        name: IntakeField {
            value: form.value("name"),
            error: None,
        },
        email: IntakeField {
            value: form.value("email"),
            error: None,
        },
        track: IntakeField {
            value: form.value("track"),
            error: None,
        },
        message: IntakeField {
            value: form.value("message"),
            error: None,
        },
        summary_error: None,
    }
}

fn validate_intake(form: &FormData) -> IntakeFormState {
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

fn intake_validation_items<'a>(state: &'a IntakeFormState) -> Vec<ValidationItem<'a>> {
    let mut items = Vec::new();

    if let Some(error) = state.name.error.as_deref() {
        items.push(ValidationItem::link("#name", error));
    }

    if let Some(error) = state.email.error.as_deref() {
        items.push(ValidationItem::link("#email", error));
    }

    if let Some(error) = state.track.error.as_deref() {
        items.push(ValidationItem::link("#track", error));
    }

    if let Some(error) = state.message.error.as_deref() {
        items.push(ValidationItem::link("#message", error));
    }

    items
}

fn app_page(title: &str, content: Markup) -> Markup {
    html_page(title)
        .head(fonts::google_font("Inter").weights(&[400, 500, 600, 700]))
        .head(assets::stylesheet(
            assets::Asset::new(APP_STYLESHEET_PATH).version(env!("CARGO_PKG_VERSION")),
        ))
        .body(html! {
            (container(html! {
                (page_shell(html! {
                    (content)
                }))
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
    let validation_items = intake_validation_items(state);
    let summary_notice = state
        .summary_error
        .as_deref()
        .map(|error| validation_summary(error, &validation_items))
        .unwrap_or_default();

    app_page(
        "ssw-rs Intake Demo",
        html! {
            (page_header(
                "Server Side Web",
                "A small intake flow, rendered on the server.",
                html! {
                    p {
                        "This example uses the current ssw-rs stack: document rendering, stable component classes, form fields, select, flash messages, CSRF protection, and an optional first-party theme stylesheet."
                    }
                },
                Some(page_actions(html! {
                    (link_button("/style-guide", "Browse the live style guide"))
                })),
            ))

            div class="demo-grid" {
                (section(stack(html! {
                    (card_header("Why this example exists", html! {
                        p {
                            "It is intentionally narrow. The goal is to pressure the current primitives in a real route flow before the framework grows more abstraction."
                        }
                    }))
                    ul class="demo-points" {
                        li { "Layout wrappers and section surfaces" }
                        li { "Field, input, textarea, and select helpers" }
                        li { "Flash messages across redirects" }
                        li { "Cookie-backed CSRF hooks" }
                    }
                    p class="ssw-card-header__body" {
                        "Use the style guide route to inspect the current primitives outside the intake flow."
                    }
                })))

                (section(html! {
                    div class="ssw-stack" {
                        @for flash in flashes {
                            (flash_notice(flash))
                        }

                        (summary_notice)

                        (card_header("Start a project", html! {
                            p {
                                "Send a short intake note. Successful submissions redirect with a flash notice; invalid ones stay on the same page with preserved values."
                            }
                        }))

                        form class="demo-form" method="post" action="/intake" {
                            (hidden_input(CSRF_FORM_FIELD, csrf_token))
                            (text_input(&name))
                            (email_input(&email))
                            (select(&track, &track_options))
                            (textarea(&message, 5))
                            (page_actions(html! {
                                (submit_button("Send request"))
                            }))
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
    let scoped_preview = scoped_css_preview();
    let breadcrumb_items = [
        BreadcrumbItem::link("/projects", "Projects"),
        BreadcrumbItem::link("/projects/northstar", "Northstar"),
        BreadcrumbItem::current("Edit brief"),
    ];
    let pagination_items = [
        PaginationItem::link("/style-guide?page=1", "Previous"),
        PaginationItem::current("1"),
        PaginationItem::link("/style-guide?page=2", "2"),
        PaginationItem::link("/style-guide?page=3", "3"),
        PaginationItem::link("/style-guide?page=2", "Next"),
    ];
    let table_rows = [
        TableRow::new(vec![
            TableCell::row_header("Northstar"),
            TableCell::new(badge_with_variant("Active", BadgeVariant::Success)),
            TableCell::new("May 28"),
        ]),
        TableRow::new(vec![
            TableCell::row_header("Acme"),
            TableCell::new(badge_with_variant("In review", BadgeVariant::Info)),
            TableCell::new("June 4"),
        ]),
        TableRow::new(vec![
            TableCell::row_header("Orbit"),
            TableCell::new(badge("Queued")),
            TableCell::new("June 11"),
        ]),
    ];
    let stat_items = [
        StatItem::new(
            "Status",
            badge_with_variant("Active", BadgeVariant::Success),
        ),
        StatItem::new("Owner", "Mina").detail("mina@northstar.example"),
        StatItem::new("Track", "Launch"),
        StatItem::new("Due", "May 28"),
    ];

    app_page(
        "Component style guide",
        html! {
            (page_header(
                "Component Preview",
                "A live style guide for the current primitives.",
                html! {
                    p {
                        "This page exists to make visual review cheap. It is not a design system yet, but it gives us a real place to inspect structure, spacing, and state styling."
                    }
                },
                Some(page_actions(html! {
                    (link_button("/", "Back to the intake demo"))
                })),
            ))

            div class="demo-grid" {
                (section(stack(html! {
                    (card_header("Notices and actions", html! {
                        p { "These are the current primitives with the optional default theme applied." }
                    }))
                    div class="demo-style-grid" {
                        (alert("Informational notice"))
                        (flash_notice(&FlashMessage::success("Successful flash message")))
                        (flash_notice(&FlashMessage::error("Error flash message")))
                        (validation_summary(
                            "Please fix the highlighted fields.",
                            &[
                                ValidationItem::link("#preview-name", "Name is required."),
                                ValidationItem::link("#preview-track", "A selection is required."),
                            ],
                        ))
                    }
                    div class="demo-inline" {
                        (button("Primary button"))
                        (button_with_variant("Secondary button", ButtonVariant::Secondary))
                    }
                })))

                (section(stack(html! {
                    (card_header("Fields and states", html! {
                        p { "Inputs, textarea, and select should remain useful without JavaScript, even when the theme is swapped out." }
                    }))
                    (text_input(&valid_name))
                    (select(&invalid_track, &options))
                    (textarea(&preview_message, 4))
                })))
            }

            div class="demo-grid" {
                (section(stack(html! {
                    (card_header("Navigation and state", html! {
                        p { "Breadcrumbs, badges, and pagination should stay readable and stable with or without app-specific CSS." }
                    }))
                    (breadcrumbs(&breadcrumb_items))
                    div class="demo-inline" {
                        (badge("Queued"))
                        (badge_with_variant("In review", BadgeVariant::Info))
                        (badge_with_variant("Shipped", BadgeVariant::Success))
                        (badge_with_variant("Blocked", BadgeVariant::Danger))
                    }
                    (pagination(&pagination_items))
                })))

                (section(stack(html! {
                    (card_header("Tabular data", html! {
                        p { "Data-heavy routes still need honest, accessible primitives without jumping straight to client-side grids." }
                    }))
                    (data_table(&["Project", "Status", "Due"], &table_rows))
                    (stat_list(&stat_items))
                })))
            }

            (section(stack(html! {
                (card_header("Scoped CSS prototype", html! {
                    p { "This is the current proof-of-concept direction for `ssw-css`: local classes, plain CSS output, and no client runtime." }
                }))
                (scoped_preview)
            })))
        },
    )
}

fn scoped_css_preview() -> Markup {
    let styles = css! {
        ".root" {
            display: grid;
            gap: 0.75 rem;
            padding: 1 rem;
            border: 1px solid var(--ssw-color-border);
            border-radius: 0.8 rem;
            background: linear-gradient(135deg, color-mix(in srgb, var(--ssw-color-surface) 92%, #eef2ff) 0%, color-mix(in srgb, var(--ssw-color-surface) 96%, #f8fafc) 100%);
        }

        ".eyebrow" {
            margin: 0;
            color: var(--ssw-color-text-muted);
            font-size: 0.75 rem;
            font-weight: 600;
            letter-spacing: 0.08 em;
            text-transform: uppercase;
        }

        ".title" {
            margin: 0;
            color: var(--ssw-color-text);
            font-size: 1.15 rem;
            font-weight: 600;
            letter-spacing: -0.02 em;
        }

        ".copy" {
            margin: 0;
            color: #52525b;
            font-size: 0.95 rem;
            line-height: 1.6;
        }

        ".aside" {
            margin: 0;
            color: var(--ssw-color-accent);
            font-size: 0.85 rem;
            font-weight: 500;
        }

        ".root:hover .title" {
            color: #1d4ed8;
        }

        @media (min-width: 56 rem) {
            ".root" {
                grid-template-columns: minmax(0, 1fr) auto;
                align-items: end;
            }
        }
    };

    html! {
        (styles.style_tag())
        article class=(styles.class("root")) {
            div {
                p class=(styles.class("eyebrow")) { "Local to this component" }
                h3 class=(styles.class("title")) { "Scoped CSS, rendered as normal CSS." }
                p class=(styles.class("copy")) {
                    "The local selector names stay small in Rust code, but the browser still receives predictable scoped classes and a regular style block."
                }
            }
            p class=(styles.class("aside")) { "No runtime injection." }
        }
    }
}

fn thanks_page(flashes: &[FlashMessage]) -> Markup {
    app_page(
        "Request sent",
        html! {
            (section(stack(html! {
                @for flash in flashes {
                    (flash_notice(flash))
                }
                (card_header("Request sent", html! {
                    p {
                        "The redirect, flash message, and success page are all coming from the current ssw-rs request model."
                    }
                }))
                (page_actions(html! {
                    (link_button("/", "Back to the intake form"))
                }))
            })))
        },
    )
}

async fn stylesheet() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(format!("{THEME_CSS}\n{APP_CSS}"))
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
    form: web::Form<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let submission = submitted_form(&request, form);

    let verified = match submission.verify_csrf() {
        Ok(verified) => verified,
        Err(invalid) => {
            let mut state = intake_state_from(invalid.data());
            state.summary_error =
                Some("Your form expired. Reload the page and try again.".to_owned());

            return unprocessable_page(
                invalid.context(),
                intake_page(
                    &state,
                    invalid.context().flashes(),
                    invalid.context().csrf_token(),
                ),
            );
        }
    };

    let state = validate_intake(verified.data());
    if state.has_errors() {
        return unprocessable_page(
            verified.context(),
            intake_page(
                &state,
                verified.context().flashes(),
                verified.context().csrf_token(),
            ),
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
            .route(APP_STYLESHEET_PATH, web::get().to(stylesheet))
    })
    .bind(address)?
    .run()
    .await
}
