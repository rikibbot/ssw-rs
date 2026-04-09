extern crate self as ssw_html;

use std::fmt::{Display, Write};

use ssw_core::{HtmlKind, HtmlResponse, Render};

pub use ssw_html_macros::html;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Markup(String);

impl Markup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn raw(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }

    pub fn text(value: impl AsRef<str>) -> Self {
        let mut markup = Self::new();
        markup.push_text(value);
        markup
    }

    pub fn push_raw(&mut self, value: impl AsRef<str>) {
        self.0.push_str(value.as_ref());
    }

    pub fn push_text(&mut self, value: impl AsRef<str>) {
        escape_into(&mut self.0, value.as_ref());
    }

    pub fn append(&mut self, markup: impl Into<Markup>) {
        self.0.push_str(&markup.into().0);
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_html_response(self, kind: HtmlKind) -> HtmlResponse {
        HtmlResponse::new(kind, self.0)
    }
}

impl Render for Markup {
    fn render_to(&self, output: &mut String) {
        output.push_str(&self.0);
    }
}

impl From<&str> for Markup {
    fn from(value: &str) -> Self {
        Self::raw(value)
    }
}

impl From<String> for Markup {
    fn from(value: String) -> Self {
        Self(value)
    }
}

pub fn document(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    let mut markup = Markup::raw(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>",
    );
    markup.push_text(title);
    markup.push_raw("</title></head><body>");
    markup.append(body);
    markup.push_raw("</body></html>");
    markup
}

pub fn fragment(body: impl Into<Markup>) -> Markup {
    body.into()
}

pub trait RenderValue {
    fn render_value(&self, markup: &mut Markup);
}

impl RenderValue for Markup {
    fn render_value(&self, markup: &mut Markup) {
        markup.append(self.clone());
    }
}

impl RenderValue for &Markup {
    fn render_value(&self, markup: &mut Markup) {
        markup.append((*self).clone());
    }
}

impl<T> RenderValue for T
where
    T: Display,
{
    fn render_value(&self, markup: &mut Markup) {
        let mut buffer = String::new();
        write!(&mut buffer, "{self}").expect("writing into String cannot fail");
        markup.push_text(buffer);
    }
}

#[doc(hidden)]
pub mod __private {
    use std::fmt::Display;

    use super::{Markup, RenderValue};

    pub fn render_value<T>(markup: &mut Markup, value: &T)
    where
        T: RenderValue + ?Sized,
    {
        value.render_value(markup);
    }

    pub fn begin_element(markup: &mut Markup, name: &str) {
        markup.push_raw("<");
        markup.push_raw(name);
    }

    pub fn push_attribute_literal(markup: &mut Markup, name: &str, value: &str) {
        markup.push_raw(" ");
        markup.push_raw(name);
        markup.push_raw("=\"");
        markup.push_text(value);
        markup.push_raw("\"");
    }

    pub fn push_attribute_expr<T>(markup: &mut Markup, name: &str, value: &T)
    where
        T: Display + ?Sized,
    {
        markup.push_raw(" ");
        markup.push_raw(name);
        markup.push_raw("=\"");
        markup.push_text(value.to_string());
        markup.push_raw("\"");
    }

    pub fn push_boolean_attribute(markup: &mut Markup, name: &str) {
        markup.push_raw(" ");
        markup.push_raw(name);
    }

    pub fn finish_open_tag(markup: &mut Markup) {
        markup.push_raw(">");
    }

    pub fn end_element(markup: &mut Markup, name: &str) {
        markup.push_raw("</");
        markup.push_raw(name);
        markup.push_raw(">");
    }
}

fn escape_into(output: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Markup, document, html};

    #[test]
    fn escapes_text_content() {
        let markup = Markup::text("<hello> & \"goodbye\"");

        assert_eq!(markup.as_str(), "&lt;hello&gt; &amp; &quot;goodbye&quot;");
    }

    #[test]
    fn renders_document_shell() {
        let page = document("Home", Markup::raw("<main>Hi</main>"));

        assert!(page.as_str().starts_with("<!DOCTYPE html>"));
        assert!(page.as_str().contains("<title>Home</title>"));
        assert!(page.as_str().contains("<main>Hi</main>"));
    }

    #[test]
    fn renders_nested_html_macro() {
        let user = "Riccardo";
        let markup = html! {
            main class="page" {
                h1 { "Hello, " (user) }
                p class="lead" { "Server-side first." }
            }
        };

        assert_eq!(
            markup.as_str(),
            "<main class=\"page\"><h1>Hello, Riccardo</h1><p class=\"lead\">Server-side first.</p></main>"
        );
    }

    #[test]
    fn supports_conditionals_and_loops() {
        let items = ["one", "two"];
        let show_footer = true;

        let markup = html! {
            section data_kind="demo" {
                ul {
                    @for item in items {
                        li { (item) }
                    }
                }
                @if show_footer {
                    footer hidden { "Done" }
                }
            }
        };

        assert_eq!(
            markup.as_str(),
            "<section data-kind=\"demo\"><ul><li>one</li><li>two</li></ul><footer hidden>Done</footer></section>"
        );
    }

    #[test]
    fn escapes_expression_values_but_keeps_markup_raw() {
        let unsafe_text = "<script>";
        let trusted = Markup::raw("<strong>trusted</strong>");

        let markup = html! {
            div {
                (unsafe_text)
                (trusted)
            }
        };

        assert_eq!(
            markup.as_str(),
            "<div>&lt;script&gt;<strong>trusted</strong></div>"
        );
    }
}
