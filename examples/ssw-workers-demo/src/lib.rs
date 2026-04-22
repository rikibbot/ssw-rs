#[cfg(target_arch = "wasm32")]
mod app {
    use ssw_components::{
        Field, container, flash_notice, hidden_input, link_button, page_actions, page_header,
        page_shell, section, stack, submit_button, textarea,
    };
    use ssw_core::{CSRF_FORM_FIELD, FlashMessage, Response as CoreResponse, TextResponse};
    use ssw_html::{Markup, html, page as html_page};
    use ssw_workers::{page_with_context, request_context, to_worker_response};
    use worker::{Context, Env, Request, Response, Result, Router, event};

    const THEME_CSS: &str = include_str!("../../../styles/ssw-theme-default.css");

    #[derive(Debug, Clone, Copy, Default)]
    struct DemoState<'a> {
        note: &'a str,
        form_error: Option<&'a str>,
        note_error: Option<&'a str>,
    }

    fn layout(title: &str, content: Markup) -> Markup {
        html_page(title)
            .body_class("app-shell")
            .head(html! {
                link rel="stylesheet" href="/theme.css";
            })
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

        layout(
            "ssw-workers demo",
            html! {
                (page_header(
                    "Cloudflare Workers",
                    "A minimal Workers adapter proof.",
                    html! {
                        p {
                            "This example keeps the scope narrow: page rendering, POST, redirect, flash, and CSRF on the Workers fetch model."
                        }
                    },
                    Some(page_actions(link_button("/thanks", "Open the success page"))),
                ))
                (stack(html! {
                    @for flash in flashes {
                        (flash_notice(flash))
                    }
                    @if state.form_error.is_some() {
                        (flash_notice(&FlashMessage::error(state.form_error.unwrap())))
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

    #[event(fetch, respond_with_errors)]
    pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
        Router::new()
            .get_async("/theme.css", |_req, _ctx| async move {
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
                                form_error: None,
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
            .run(req, env)
            .await
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn host_build_stub() {}
