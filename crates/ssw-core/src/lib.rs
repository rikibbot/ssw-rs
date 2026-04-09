//! Core rendering and response primitives shared by `ssw-rs` crates.

use std::borrow::Cow;

/// Distinguishes full HTML documents from partial fragments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlKind {
    /// A complete HTML document, typically including `<!DOCTYPE html>`.
    Document,
    /// A partial HTML fragment intended to be embedded into an existing page.
    Fragment,
}

/// A value that can render itself into HTML or text output.
pub trait Render {
    /// Appends the rendered representation to the provided output buffer.
    fn render_to(&self, output: &mut String);

    /// Renders the value into an owned string.
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

/// A rendered HTML response body plus its document or fragment kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlResponse {
    kind: HtmlKind,
    body: String,
}

impl HtmlResponse {
    /// Creates a new HTML response with the given kind and body.
    pub fn new(kind: HtmlKind, body: impl Into<String>) -> Self {
        Self {
            kind,
            body: body.into(),
        }
    }

    /// Creates a document response from an owned body.
    pub fn document(body: impl Into<String>) -> Self {
        Self::new(HtmlKind::Document, body)
    }

    /// Creates a fragment response from an owned body.
    pub fn fragment(body: impl Into<String>) -> Self {
        Self::new(HtmlKind::Fragment, body)
    }

    /// Renders a value and stores the resulting HTML body.
    pub fn from_rendered(kind: HtmlKind, value: impl Render) -> Self {
        Self::new(kind, value.render())
    }

    /// Returns whether this response is a document or fragment.
    pub fn kind(&self) -> HtmlKind {
        self.kind
    }

    /// Returns the rendered body.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Consumes the response and returns the rendered body.
    pub fn into_body(self) -> String {
        self.body
    }
}

/// A plain text response body plus an explicit content type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextResponse {
    body: String,
    content_type: Cow<'static, str>,
}

impl TextResponse {
    /// Creates a new text response with an explicit content type.
    pub fn new(body: impl Into<String>, content_type: impl Into<Cow<'static, str>>) -> Self {
        Self {
            body: body.into(),
            content_type: content_type.into(),
        }
    }

    /// Creates a UTF-8 plain text response.
    pub fn plain(body: impl Into<String>) -> Self {
        Self::new(body, "text/plain; charset=utf-8")
    }

    /// Returns the body content.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the response content type.
    pub fn content_type(&self) -> &str {
        &self.content_type
    }
}

/// The redirect status code family to use for a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectKind {
    /// `303 See Other`
    SeeOther,
    /// `307 Temporary Redirect`
    Temporary,
    /// `308 Permanent Redirect`
    Permanent,
}

/// A redirect target plus its semantic redirect kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    kind: RedirectKind,
    location: String,
}

impl Redirect {
    /// Creates a new redirect with an explicit redirect kind.
    pub fn new(kind: RedirectKind, location: impl Into<String>) -> Self {
        Self {
            kind,
            location: location.into(),
        }
    }

    /// Creates a `303 See Other` redirect.
    pub fn see_other(location: impl Into<String>) -> Self {
        Self::new(RedirectKind::SeeOther, location)
    }

    /// Creates a `307 Temporary Redirect`.
    pub fn temporary(location: impl Into<String>) -> Self {
        Self::new(RedirectKind::Temporary, location)
    }

    /// Creates a `308 Permanent Redirect`.
    pub fn permanent(location: impl Into<String>) -> Self {
        Self::new(RedirectKind::Permanent, location)
    }

    /// Returns the redirect kind.
    pub fn kind(&self) -> RedirectKind {
        self.kind
    }

    /// Returns the redirect location.
    pub fn location(&self) -> &str {
        &self.location
    }
}

/// A backend-agnostic response enum used across `ssw-rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// An HTML document or fragment response.
    Html(HtmlResponse),
    /// A text response.
    Text(TextResponse),
    /// A redirect response.
    Redirect(Redirect),
}

impl Response {
    /// Creates an HTML response from a kind and body.
    pub fn html(kind: HtmlKind, body: impl Into<String>) -> Self {
        Self::Html(HtmlResponse::new(kind, body))
    }

    /// Renders a value into an HTML response.
    pub fn html_rendered(kind: HtmlKind, value: impl Render) -> Self {
        Self::Html(HtmlResponse::from_rendered(kind, value))
    }

    /// Creates a UTF-8 plain text response.
    pub fn text(body: impl Into<String>) -> Self {
        Self::Text(TextResponse::plain(body))
    }

    /// Creates a `303 See Other` redirect response.
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
