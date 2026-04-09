use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlKind {
    Document,
    Fragment,
}

pub trait Render {
    fn render_to(&self, output: &mut String);

    fn render(&self) -> String {
        let mut output = String::new();
        self.render_to(&mut output);
        output
    }
}

impl Render for str {
    fn render_to(&self, output: &mut String) {
        output.push_str(self);
    }
}

impl Render for String {
    fn render_to(&self, output: &mut String) {
        output.push_str(self);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlResponse {
    kind: HtmlKind,
    body: String,
}

impl HtmlResponse {
    pub fn new(kind: HtmlKind, body: impl Into<String>) -> Self {
        Self {
            kind,
            body: body.into(),
        }
    }

    pub fn document(body: impl Into<String>) -> Self {
        Self::new(HtmlKind::Document, body)
    }

    pub fn fragment(body: impl Into<String>) -> Self {
        Self::new(HtmlKind::Fragment, body)
    }

    pub fn from_rendered(kind: HtmlKind, value: impl Render) -> Self {
        Self::new(kind, value.render())
    }

    pub fn kind(&self) -> HtmlKind {
        self.kind
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn into_body(self) -> String {
        self.body
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextResponse {
    body: String,
    content_type: Cow<'static, str>,
}

impl TextResponse {
    pub fn new(body: impl Into<String>, content_type: impl Into<Cow<'static, str>>) -> Self {
        Self {
            body: body.into(),
            content_type: content_type.into(),
        }
    }

    pub fn plain(body: impl Into<String>) -> Self {
        Self::new(body, "text/plain; charset=utf-8")
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectKind {
    SeeOther,
    Temporary,
    Permanent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    kind: RedirectKind,
    location: String,
}

impl Redirect {
    pub fn new(kind: RedirectKind, location: impl Into<String>) -> Self {
        Self {
            kind,
            location: location.into(),
        }
    }

    pub fn see_other(location: impl Into<String>) -> Self {
        Self::new(RedirectKind::SeeOther, location)
    }

    pub fn temporary(location: impl Into<String>) -> Self {
        Self::new(RedirectKind::Temporary, location)
    }

    pub fn permanent(location: impl Into<String>) -> Self {
        Self::new(RedirectKind::Permanent, location)
    }

    pub fn kind(&self) -> RedirectKind {
        self.kind
    }

    pub fn location(&self) -> &str {
        &self.location
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Html(HtmlResponse),
    Text(TextResponse),
    Redirect(Redirect),
}

impl Response {
    pub fn html(kind: HtmlKind, body: impl Into<String>) -> Self {
        Self::Html(HtmlResponse::new(kind, body))
    }

    pub fn html_rendered(kind: HtmlKind, value: impl Render) -> Self {
        Self::Html(HtmlResponse::from_rendered(kind, value))
    }

    pub fn text(body: impl Into<String>) -> Self {
        Self::Text(TextResponse::plain(body))
    }

    pub fn redirect(location: impl Into<String>) -> Self {
        Self::Redirect(Redirect::see_other(location))
    }
}

#[cfg(test)]
mod tests {
    use super::{HtmlKind, HtmlResponse, Render, Response};

    struct Greeting<'a>(&'a str);

    impl Render for Greeting<'_> {
        fn render_to(&self, output: &mut String) {
            output.push_str("Hello, ");
            output.push_str(self.0);
            output.push('!');
        }
    }

    #[test]
    fn renders_from_trait() {
        let response = HtmlResponse::from_rendered(HtmlKind::Fragment, Greeting("ssw"));

        assert_eq!(response.body(), "Hello, ssw!");
        assert_eq!(response.kind(), HtmlKind::Fragment);
    }

    #[test]
    fn wraps_rendered_html_response() {
        let response = Response::html_rendered(HtmlKind::Document, Greeting("world"));

        match response {
            Response::Html(html) => assert_eq!(html.body(), "Hello, world!"),
            _ => panic!("expected html response"),
        }
    }
}
