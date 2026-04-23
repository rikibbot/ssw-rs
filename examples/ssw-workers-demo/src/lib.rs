//! Example Cloudflare Workers app for the minimal `ssw-workers` adapter slice.
//!
//! This module exists to pressure:
//! - response conversion under the Workers runtime
//! - request-context, flash, and CSRF hooks
//! - fragment rendering and HTML 404 pages
//! - explicit asset routes under a non-Actix backend

#[cfg(target_arch = "wasm32")]
mod app {
    use ssw_components::{
        Field, ValidationItem, container, flash_notice, hidden_input, link_button, page_actions,
        page_header, page_shell, section, stack, submit_button, textarea, validation_summary,
    };
    use ssw_core::{
        CSRF_FORM_FIELD, FlashMessage, HtmlKind, Response as CoreResponse, TextResponse,
    };
    use ssw_html::{Markup, assets, html, page as html_page};
    use ssw_workers::{fragment, page_with_context, request_context, to_worker_response};
    use worker::{Context, Env, Request, Response, Result, Router, event};

    const THEME_CSS: &str = include_str!("../../../styles/ssw-theme-default.css");
    const THEME_STYLESHEET_PATH: &str = "/assets/theme.css";

    #[derive(Debug, Clone, Copy, Default)]
    struct DemoState<'a> {
        note: &'a str,
        form_error: Option<&'a str>,
        note_error: Option<&'a str>,
    }

    fn validation_summary_items(state: &DemoState<'_>) -> Vec<ValidationItem<'_>> {
        let mut items = Vec::new();

        if let Some(error) = state.note_error {
            items.push(ValidationItem::link("#note", error));
        }

        items
    }

    fn layout(title: &str, content: Markup) -> Markup {
        html_page(title)
            .body_class("app-shell")
            .head(assets::stylesheet(
                assets::Asset::new(THEME_STYLESHEET_PATH).version(env!("CARGO_PKG_VERSION")),
            ))
            .body(page_shell(container(html! {
                main {
                    (content)
                }
            })))
            .render()
    }

    fn form_page(state: &DemoState<'_>, flashes: &[FlashMessage], csrf_token: &str) -> Markup {
        let note = Field::new("note", "note", "Delivery note")
            .value(state.note)
            .error(state.note_error)
            .required(true);
        let validation_items = validation_summary_items(state);

        layout(
            "ssw-workers demo",
            html! {
                (page_header(
                    "Cloudflare Workers",
                    "A minimal Workers adapter proof.",
                    html! {
                        p {
                            "This example keeps the scope narrow: page rendering, fragments, POST, redirect, flash, and CSRF on the Workers fetch model."
                        }
                    },
                    Some(page_actions(html! {
                        (link_button("/preview?note=Rendered%20at%20the%20edge", "Open the preview fragment"))
                        (link_button("/thanks", "Open the success page"))
                    })),
                ))
                (stack(html! {
                    @for flash in flashes {
                        (flash_notice(flash))
                    }
                    @if state.form_error.is_some() {
                        (validation_summary(state.form_error.unwrap(), &validation_items))
                    }
                    (section(html! {
                        form method="post" action="/" {
                            (hidden_input(CSRF_FORM_FIELD, csrf_token))
                            (textarea(&note, 5))
                            (submit_button("Send note"))
                        }
                    }))
                }))
            },
        )
    }

    fn preview_fragment(note: &str) -> Markup {
        let body = if note.trim().is_empty() {
            "Workers can return partial HTML without wrapping it in a full document."
        } else {
            note
        };

        html! {
            div class="ssw-fragment-preview" {
                strong { "Fragment preview" }
                p { (body) }
            }
        }
    }

    fn thanks_page(flashes: &[FlashMessage]) -> Markup {
        layout(
            "ssw-workers demo",
            html! {
                (page_header(
                    "Cloudflare Workers",
                    "Submission complete.",
                    html! {
                        p {
                            "The success page is rendered from a Worker and clears the flash cookie after use."
                        }
                    },
                    Some(page_actions(link_button("/", "Back to the form"))),
                ))
                (stack(html! {
                    @for flash in flashes {
                        (flash_notice(flash))
                    }
                }))
            },
        )
    }

    fn not_found_page(path: &str) -> Markup {
        layout(
            "Not found",
            html! {
                (page_header(
                    "Cloudflare Workers",
                    "Page not found.",
                    html! {
                        p {
                            "This route uses the shared response model to render a normal HTML document shell with a 404 status."
                        }
                    },
                    Some(page_actions(link_button("/", "Back to the form"))),
                ))
                (section(html! {
                    p {
                        "The requested path was "
                        code { (path) }
                        "."
                    }
                    p {
                        "This now works through "
                        code { "ssw-core::Response" }
                        " rather than a Worker-specific fallback helper."
                    }
                }))
            },
        )
    }

    #[event(fetch, respond_with_errors)]
    pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
        Router::new()
            .get_async(THEME_STYLESHEET_PATH, |_req, _ctx| async move {
                to_worker_response(CoreResponse::Text(TextResponse::new(
                    THEME_CSS,
                    "text/css; charset=utf-8",
                )))
            })
            .get_async("/", |req, _ctx| async move {
                let context = request_context(&req)?;
                page_with_context(
                    &context,
                    form_page(
                        &DemoState::default(),
                        context.flashes(),
                        context.csrf_token(),
                    ),
                )
            })
            .get_async("/preview", |req, _ctx| async move {
                let note = req
                    .url()?
                    .query_pairs()
                    .find_map(|(key, value)| (key == "note").then(|| value.into_owned()))
                    .unwrap_or_default();

                fragment(preview_fragment(&note))
            })
            .post_async("/", |mut req, _ctx| async move {
                let context = request_context(&req)?;
                let form = req.form_data().await?;
                let note = form.get_field("note").unwrap_or_default();

                if context
                    .verify_csrf(form.get_field(CSRF_FORM_FIELD).as_deref())
                    .is_err()
                {
                    return page_with_context(
                        &context,
                        form_page(
                            &DemoState {
                                note: &note,
                                form_error: Some(
                                    "Your form expired. Reload the page and try again.",
                                ),
                                note_error: None,
                            },
                            context.flashes(),
                            context.csrf_token(),
                        ),
                    );
                }

                if note.trim().is_empty() {
                    return page_with_context(
                        &context,
                        form_page(
                            &DemoState {
                                note: &note,
                                form_error: Some("Please fix the highlighted fields."),
                                note_error: Some("Add a short note before submitting."),
                            },
                            context.flashes(),
                            context.csrf_token(),
                        ),
                    );
                }

                to_worker_response(CoreResponse::redirect_with_flash(
                    "/thanks",
                    FlashMessage::success("Saved from the Workers adapter."),
                ))
            })
            .get_async("/thanks", |req, _ctx| async move {
                let context = request_context(&req)?;
                page_with_context(&context, thanks_page(context.flashes()))
            })
            .or_else_any_method_async("/*path", |req, ctx| async move {
                let path = format!("/{}", ctx.param("path").cloned().unwrap_or_default());
                let context = request_context(&req)?;

                context.apply(to_worker_response(
                    CoreResponse::html_rendered_with_status(
                        404,
                        HtmlKind::Document,
                        not_found_page(&path),
                    ),
                )?)
            })
            .run(req, env)
            .await
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn host_build_stub() {}
