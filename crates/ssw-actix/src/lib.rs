use actix_web::HttpResponse;
use actix_web::http::StatusCode;
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

pub fn render_html(kind: HtmlKind, view: impl Render) -> HttpResponse {
    to_http_response(Response::html_rendered(kind, view))
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
    use ssw_core::{HtmlKind, Response};
    use ssw_html::Markup;

    use super::{render_html, to_http_response};

    #[actix_web::test]
    async fn converts_html_response() {
        let response = to_http_response(Response::html(HtmlKind::Document, "<h1>home</h1>"));

        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(actix_web::http::header::CONTENT_TYPE)
                .unwrap(),
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
}
