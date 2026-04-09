use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder};
use ssw_core::{HtmlKind, RedirectKind, Render, Response};

pub struct ActixResponse(pub Response);

impl From<Response> for ActixResponse {
    fn from(response: Response) -> Self {
        Self(response)
    }
}

impl ActixResponse {
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

pub fn render_html(kind: HtmlKind, view: impl Render) -> HttpResponse {
    to_http_response(Response::html_rendered(kind, view))
}

pub fn page(view: impl Render) -> HttpResponse {
    render_html(HtmlKind::Document, view)
}

pub fn fragment(view: impl Render) -> HttpResponse {
    render_html(HtmlKind::Fragment, view)
}

pub fn redirect(location: impl Into<String>) -> HttpResponse {
    to_http_response(Response::redirect(location))
}

pub fn to_http_response(response: Response) -> HttpResponse {
    match response {
        Response::Html(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html.into_body()),
        Response::Text(text) => HttpResponse::Ok()
            .content_type(text.content_type())
            .body(text.body().to_owned()),
        Response::Redirect(redirect) => HttpResponse::build(status_for_redirect(redirect.kind()))
            .insert_header(("Location", redirect.location().to_owned()))
            .finish(),
    }
}

fn status_for_redirect(kind: RedirectKind) -> StatusCode {
    match kind {
        RedirectKind::SeeOther => StatusCode::SEE_OTHER,
        RedirectKind::Temporary => StatusCode::TEMPORARY_REDIRECT,
        RedirectKind::Permanent => StatusCode::PERMANENT_REDIRECT,
    }
}

#[cfg(test)]
mod tests {
    use actix_web::body::to_bytes;
    use actix_web::http::{StatusCode, header};
    use actix_web::test;
    use actix_web::{App, HttpResponse, web};
    use ssw_core::{HtmlKind, Response};
    use ssw_html::{Markup, html, page as html_page};

    use super::{ActixResponse, fragment, page, redirect, render_html, to_http_response};

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
    async fn renders_from_core_trait() {
        let response = render_html(HtmlKind::Fragment, Markup::text("Hello from Actix"));
        let body = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body, "Hello from Actix");
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
}
