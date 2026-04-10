use std::collections::HashMap;

use actix_web::http::header;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use ssw_actix::{CSRF_FORM_FIELD, page_with_context, request_context, to_http_response};
use ssw_components::{
    Field, NavItem, SelectOption, button_with_variant, card_header, container, email_input,
    empty_state, flash_notice, hidden_input, page_actions, page_header, page_shell, section,
    select, stack, submit_button, text_input, textarea, top_nav,
};
use ssw_core::{FlashMessage, Response};
use ssw_html::{Markup, fonts, html, page as html_page};

const THEME_CSS: &str = include_str!("../../../styles/ssw-theme-default.css");
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

.demo-link {
  color: #18181b;
  font-weight: 500;
  text-decoration: underline;
  text-underline-offset: 0.16em;
}

.demo-link:hover {
  color: #09090b;
}

.projects-grid {
  display: grid;
  gap: 1rem;
  align-items: start;
}

.projects-list {
  display: grid;
  gap: 0.85rem;
}

.project-card {
  display: grid;
  gap: 0.65rem;
  padding: 1rem;
  border: 1px solid var(--ssw-color-border);
  border-radius: var(--ssw-radius-md);
  background: color-mix(in srgb, var(--ssw-color-surface) 84%, white);
  text-decoration: none;
}

.project-card:hover {
  border-color: var(--ssw-color-border-strong);
  background: var(--ssw-color-surface);
}

.project-card__eyebrow,
.project-meta dt {
  margin: 0;
  color: var(--ssw-color-text-muted);
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.project-card__title {
  margin: 0;
  color: var(--ssw-color-text);
  font-size: 1.05rem;
  font-weight: 600;
  line-height: 1.15;
  letter-spacing: -0.02em;
}

.project-card__summary,
.project-meta dd,
.detail-copy {
  margin: 0;
  color: #52525b;
  font-size: 0.95rem;
  line-height: 1.6;
}

.project-meta {
  display: grid;
  gap: 0.85rem;
}

.project-meta__row {
  display: grid;
  gap: 0.2rem;
}

.project-form {
  display: grid;
  gap: 0.9rem;
}

@media (min-width: 64rem) {
  .projects-grid {
    grid-template-columns: minmax(0, 1.05fr) minmax(18rem, 0.95fr);
  }
}
"#;

#[derive(Debug, Clone, Copy)]
struct Project {
    slug: &'static str,
    title: &'static str,
    client: &'static str,
    summary: &'static str,
    status: &'static str,
    track: &'static str,
    owner: &'static str,
    due: &'static str,
    contact_email: &'static str,
}

const PROJECTS: [Project; 3] = [
    Project {
        slug: "northstar",
        title: "Northstar launch sprint",
        client: "Northstar Health",
        summary: "Marketing launch work, editorial pages, intake forms, and a calmer CMS handoff.",
        status: "active",
        track: "launch",
        owner: "Mina",
        due: "May 28",
        contact_email: "mina@northstar.example",
    },
    Project {
        slug: "acme-migration",
        title: "Acme migration review",
        client: "Acme",
        summary: "Server-rendered migration planning, route mapping, and cutover readiness checks.",
        status: "review",
        track: "migration",
        owner: "Theo",
        due: "June 4",
        contact_email: "theo@acme.example",
    },
    Project {
        slug: "orbit-audit",
        title: "Orbit architecture audit",
        client: "Orbit",
        summary: "A short architecture engagement focused on rendering boundaries, forms, and asset strategy.",
        status: "queued",
        track: "audit",
        owner: "Aya",
        due: "June 11",
        contact_email: "aya@orbit.example",
    },
];

#[derive(Debug, Clone, Default)]
struct EditField {
    value: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct EditFormState {
    title: EditField,
    owner_email: EditField,
    track: EditField,
    status: EditField,
    summary: EditField,
    summary_error: Option<String>,
}

impl EditFormState {
    fn has_errors(&self) -> bool {
        self.summary_error.is_some()
    }
}

fn project_by_slug(slug: &str) -> Option<Project> {
    PROJECTS
        .iter()
        .copied()
        .find(|project| project.slug == slug)
}

fn nav_items(current: &str) -> [NavItem<'static>; 2] {
    [
        NavItem::new("/projects", "Projects").current(current == "projects"),
        NavItem::new("/projects/archive", "Archive").current(current == "archive"),
    ]
}

fn status_options() -> [SelectOption<'static>; 4] {
    [
        SelectOption::new("", "Choose a status"),
        SelectOption::new("queued", "Queued"),
        SelectOption::new("active", "Active"),
        SelectOption::new("review", "In review"),
    ]
}

fn track_options() -> [SelectOption<'static>; 4] {
    [
        SelectOption::new("", "Choose a track"),
        SelectOption::new("launch", "Launch sprint"),
        SelectOption::new("migration", "Migration"),
        SelectOption::new("audit", "Architecture audit"),
    ]
}

fn project_edit_state(project: Project) -> EditFormState {
    EditFormState {
        title: EditField {
            value: project.title.to_owned(),
            error: None,
        },
        owner_email: EditField {
            value: project.contact_email.to_owned(),
            error: None,
        },
        track: EditField {
            value: project.track.to_owned(),
            error: None,
        },
        status: EditField {
            value: project.status.to_owned(),
            error: None,
        },
        summary: EditField {
            value: project.summary.to_owned(),
            error: None,
        },
        summary_error: None,
    }
}

fn edit_state_from_form(form: &HashMap<String, String>, project: Project) -> EditFormState {
    EditFormState {
        title: EditField {
            value: form
                .get("title")
                .cloned()
                .unwrap_or_else(|| project.title.to_owned()),
            error: None,
        },
        owner_email: EditField {
            value: form
                .get("owner_email")
                .cloned()
                .unwrap_or_else(|| project.contact_email.to_owned()),
            error: None,
        },
        track: EditField {
            value: form
                .get("track")
                .cloned()
                .unwrap_or_else(|| project.track.to_owned()),
            error: None,
        },
        status: EditField {
            value: form
                .get("status")
                .cloned()
                .unwrap_or_else(|| project.status.to_owned()),
            error: None,
        },
        summary: EditField {
            value: form
                .get("summary")
                .cloned()
                .unwrap_or_else(|| project.summary.to_owned()),
            error: None,
        },
        summary_error: None,
    }
}

fn validate_edit_form(form: &HashMap<String, String>, project: Project) -> EditFormState {
    let mut state = edit_state_from_form(form, project);

    if state.title.value.trim().is_empty() {
        state.title.error = Some("Title is required.".to_owned());
    }

    if state.owner_email.value.trim().is_empty() {
        state.owner_email.error = Some("Owner email is required.".to_owned());
    } else if !state.owner_email.value.contains('@') {
        state.owner_email.error = Some("Owner email must look valid.".to_owned());
    }

    if state.track.value.trim().is_empty() {
        state.track.error = Some("Pick a project track.".to_owned());
    }

    if state.status.value.trim().is_empty() {
        state.status.error = Some("Pick a project status.".to_owned());
    }

    if state.summary.value.trim().len() < 16 {
        state.summary.error = Some("Summary should give a little more context.".to_owned());
    }

    if state.title.error.is_some()
        || state.owner_email.error.is_some()
        || state.track.error.is_some()
        || state.status.error.is_some()
        || state.summary.error.is_some()
    {
        state.summary_error = Some("Please fix the highlighted fields.".to_owned());
    }

    state
}

fn app_page(title: &str, nav_current: &str, body: Markup) -> Markup {
    html_page(title)
        .head(fonts::google_font("Inter").weights(&[400, 500, 600, 700]))
        .head(html! {
            link rel="stylesheet" href="/app.css";
        })
        .body(html! {
            (container(page_shell(html! {
                (top_nav("/", "Server Side Web", &nav_items(nav_current)))
                (body)
            })))
        })
        .render()
}

fn projects_page(flashes: &[FlashMessage]) -> Markup {
    app_page(
        "Projects",
        "projects",
        html! {
            (page_header(
                "Project Studio",
                "A second example with list, detail, archive, and edit flows.",
                html! {
                    p {
                        "This example pressures navigation, empty states, page shell, actions, redirects, and the current form primitives in a more app-shaped flow."
                    }
                },
                Some(page_actions(html! {
                    a class="demo-link" href="/projects/archive" { "View archive" }
                })),
            ))

            div class="projects-grid" {
                (section(stack(html! {
                    @for flash in flashes {
                        (flash_notice(flash))
                    }
                    (card_header("Active projects", html! {
                        p { "Open a project to inspect a detail page and a server-rendered edit flow." }
                    }))
                    div class="projects-list" {
                        @for project in PROJECTS {
                            a class="project-card" href=(format!("/projects/{}", project.slug)) {
                                p class="project-card__eyebrow" { (project.client) }
                                h2 class="project-card__title" { (project.title) }
                                p class="project-card__summary" { (project.summary) }
                            }
                        }
                    }
                })))

                (section(stack(html! {
                    (card_header("Why this example matters", html! {
                        p { "The intake flow was good for forms. This one is better for page shell and app-level composition." }
                    }))
                    dl class="project-meta" {
                        div class="project-meta__row" {
                            dt { "Pages" }
                            dd { "List, detail, archive, and edit" }
                        }
                        div class="project-meta__row" {
                            dt { "Primitives under pressure" }
                            dd { "Top nav, page header, action rows, card headers, empty states, field helpers" }
                        }
                        div class="project-meta__row" {
                            dt { "Intentional limit" }
                            dd { "No persistence yet, so the focus stays on rendering and request flow." }
                        }
                    }
                })))
            }
        },
    )
}

fn archive_page() -> Markup {
    app_page(
        "Archive",
        "archive",
        html! {
            (page_header(
                "Project Studio",
                "Archive",
                html! {
                    p {
                        "An empty state is a first-class screen, not just a missing list item."
                    }
                },
                Some(page_actions(html! {
                    a class="demo-link" href="/projects" { "Back to active projects" }
                })),
            ))

            (empty_state(
                "No archived projects yet",
                html! {
                    p {
                        "This route exists to pressure the empty-state primitive in a real page shell. When the next example grows server state, this should still hold up cleanly."
                    }
                },
                Some(page_actions(html! {
                    a class="demo-link" href="/projects" { "Browse active work" }
                })),
            ))
        },
    )
}

fn project_detail_page(project: Project, flashes: &[FlashMessage]) -> Markup {
    app_page(
        project.title,
        "projects",
        html! {
            (page_header(
                project.client,
                project.title,
                html! {
                    p { (project.summary) }
                },
                Some(page_actions(html! {
                    a class="demo-link" href="/projects" { "All projects" }
                    a class="demo-link" href=(format!("/projects/{}/edit", project.slug)) { "Edit brief" }
                })),
            ))

            div class="projects-grid" {
                (section(stack(html! {
                    @for flash in flashes {
                        (flash_notice(flash))
                    }
                    (card_header("Project brief", html! {
                        p { "A detail page lets us pressure richer page composition than the intake form alone." }
                    }))
                    p class="detail-copy" { (project.summary) }
                    p class="detail-copy" {
                        "The page is static by design. What matters here is that the shell, nav, actions, and surrounding content still read like a real app."
                    }
                })))

                (section(stack(html! {
                    (card_header("Project metadata", html! {
                        p { "This is intentionally boring data, because the layout should still feel deliberate." }
                    }))
                    dl class="project-meta" {
                        div class="project-meta__row" {
                            dt { "Status" }
                            dd { (project.status) }
                        }
                        div class="project-meta__row" {
                            dt { "Track" }
                            dd { (project.track) }
                        }
                        div class="project-meta__row" {
                            dt { "Owner" }
                            dd { (project.owner) }
                        }
                        div class="project-meta__row" {
                            dt { "Due" }
                            dd { (project.due) }
                        }
                        div class="project-meta__row" {
                            dt { "Contact" }
                            dd { (project.contact_email) }
                        }
                    }
                })))
            }
        },
    )
}

fn project_edit_page(
    project: Project,
    state: &EditFormState,
    flashes: &[FlashMessage],
    csrf_token: &str,
) -> Markup {
    let title = Field::new("title", "title", "Project title")
        .value(state.title.value.as_str())
        .error(state.title.error.as_deref())
        .required(true);
    let owner_email = Field::new("owner-email", "owner_email", "Owner email")
        .value(state.owner_email.value.as_str())
        .error(state.owner_email.error.as_deref())
        .required(true);
    let track = Field::new("track", "track", "Track")
        .value(state.track.value.as_str())
        .error(state.track.error.as_deref())
        .required(true);
    let status = Field::new("status", "status", "Status")
        .value(state.status.value.as_str())
        .error(state.status.error.as_deref())
        .required(true);
    let summary = Field::new("summary", "summary", "Project summary")
        .value(state.summary.value.as_str())
        .error(state.summary.error.as_deref())
        .required(true);
    let status_options = status_options();
    let track_options = track_options();

    app_page(
        "Edit project",
        "projects",
        html! {
            (page_header(
                project.client,
                "Edit project brief",
                html! {
                    p {
                        "This route pressures the current SSR form layer in a page that is not just an intake flow."
                    }
                },
                Some(page_actions(html! {
                    a class="demo-link" href=(format!("/projects/{}", project.slug)) { "Back to project" }
                })),
            ))

            div class="projects-grid" {
                (section(stack(html! {
                    @for flash in flashes {
                        (flash_notice(flash))
                    }

                    @if state.summary_error.is_some() {
                        (flash_notice(&FlashMessage::error(
                            state.summary_error.as_deref().unwrap(),
                        )))
                    }

                    (card_header("Edit project", html! {
                        p { "Successful submissions still redirect with a flash; invalid ones stay on the same page with preserved values." }
                    }))

                    form class="project-form" method="post" action=(format!("/projects/{}/edit", project.slug)) {
                        (hidden_input(CSRF_FORM_FIELD, csrf_token))
                        (text_input(&title))
                        (email_input(&owner_email))
                        (select(&track, &track_options))
                        (select(&status, &status_options))
                        (textarea(&summary, 6))
                        (page_actions(html! {
                            (submit_button("Review update"))
                            a class="demo-link" href=(format!("/projects/{}", project.slug)) { "Cancel" }
                        }))
                    }
                })))

                (section(stack(html! {
                    (card_header("Why it does not persist", html! {
                        p { "The goal is to verify form and page ergonomics before the examples pick a persistence story." }
                    }))
                    p class="detail-copy" {
                        "A successful POST redirects back to the detail page with a flash notice, but the example does not mutate shared state yet. That limitation is intentional."
                    }
                    (page_actions(html! {
                        (button_with_variant("Server-owned state next", ssw_components::ButtonVariant::Secondary))
                    }))
                })))
            }
        },
    )
}

async fn stylesheet() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(format!("{THEME_CSS}\n{APP_CSS}"))
}

async fn root() -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/projects"))
        .finish()
}

async fn projects(request: HttpRequest) -> HttpResponse {
    let context = request_context(&request);
    page_with_context(&context, projects_page(context.flashes()))
}

async fn archive() -> HttpResponse {
    to_http_response(Response::html_rendered(
        ssw_core::HtmlKind::Document,
        archive_page(),
    ))
}

async fn project_detail(request: HttpRequest, slug: web::Path<String>) -> HttpResponse {
    let context = request_context(&request);
    let Some(project) = project_by_slug(&slug) else {
        return HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body("project not found");
    };

    page_with_context(&context, project_detail_page(project, context.flashes()))
}

async fn project_edit_get(request: HttpRequest, slug: web::Path<String>) -> HttpResponse {
    let context = request_context(&request);
    let Some(project) = project_by_slug(&slug) else {
        return HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body("project not found");
    };

    page_with_context(
        &context,
        project_edit_page(
            project,
            &project_edit_state(project),
            context.flashes(),
            context.csrf_token(),
        ),
    )
}

async fn project_edit_post(
    request: HttpRequest,
    slug: web::Path<String>,
    form: web::Form<HashMap<String, String>>,
) -> HttpResponse {
    let context = request_context(&request);
    let Some(project) = project_by_slug(&slug) else {
        return HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body("project not found");
    };

    if context
        .verify_csrf(form.get(CSRF_FORM_FIELD).map(String::as_str))
        .is_err()
    {
        let mut state = edit_state_from_form(&form, project);
        state.summary_error = Some("Your form expired. Reload the page and try again.".to_owned());

        return page_with_context(
            &context,
            project_edit_page(project, &state, context.flashes(), context.csrf_token()),
        );
    }

    let state = validate_edit_form(&form, project);
    if state.has_errors() {
        return page_with_context(
            &context,
            project_edit_page(project, &state, context.flashes(), context.csrf_token()),
        );
    }

    to_http_response(Response::redirect_with_flash(
        format!("/projects/{}", project.slug),
        FlashMessage::success(
            "Project brief reviewed server-side. Persistence is intentionally out of scope for this demo.",
        ),
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3002);
    let address = format!("127.0.0.1:{port}");

    println!("ssw-projects-demo listening on http://{address}");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(root))
            .route("/projects", web::get().to(projects))
            .route("/projects/archive", web::get().to(archive))
            .route("/projects/{slug}", web::get().to(project_detail))
            .route("/projects/{slug}/edit", web::get().to(project_edit_get))
            .route("/projects/{slug}/edit", web::post().to(project_edit_post))
            .route("/app.css", web::get().to(stylesheet))
    })
    .bind(address)?
    .run()
    .await
}
