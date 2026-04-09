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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    title: String,
    lang: String,
    head: Markup,
    body: Markup,
    body_class: Option<String>,
}

impl Document {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lang: "en".to_owned(),
            head: Markup::new(),
            body: Markup::new(),
            body_class: None,
        }
    }

    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    pub fn head(mut self, markup: impl Into<Markup>) -> Self {
        self.head.append(markup);
        self
    }

    pub fn body(mut self, markup: impl Into<Markup>) -> Self {
        self.body = markup.into();
        self
    }

    pub fn body_class(mut self, class: impl Into<String>) -> Self {
        self.body_class = Some(class.into());
        self
    }

    pub fn render(self) -> Markup {
        let mut markup = Markup::raw("<!DOCTYPE html>");
        markup.append(html! {
            html lang=(self.lang) {
                head {
                    meta charset="utf-8";
                    meta name="viewport" content="width=device-width, initial-scale=1";
                    title { (self.title) }
                    (self.head)
                }
                body class=(self.body_class) {
                    (self.body)
                }
            }
        });
        markup
    }
}

pub fn page(title: impl Into<String>) -> Document {
    Document::new(title)
}

pub fn document(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    page(title.as_ref()).body(body).render()
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

pub trait AttributeValue {
    fn render_attribute_value(&self, markup: &mut Markup, name: &str);
}

impl<T> AttributeValue for Option<T>
where
    T: AttributeValue,
{
    fn render_attribute_value(&self, markup: &mut Markup, name: &str) {
        if let Some(value) = self {
            value.render_attribute_value(markup, name);
        }
    }
}

macro_rules! impl_attribute_value_display {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl AttributeValue for $ty {
                fn render_attribute_value(&self, markup: &mut Markup, name: &str) {
                    push_attribute(markup, name, self);
                }
            }
        )+
    };
}

impl AttributeValue for String {
    fn render_attribute_value(&self, markup: &mut Markup, name: &str) {
        push_attribute(markup, name, self);
    }
}

impl AttributeValue for str {
    fn render_attribute_value(&self, markup: &mut Markup, name: &str) {
        push_attribute(markup, name, self);
    }
}

impl AttributeValue for &str {
    fn render_attribute_value(&self, markup: &mut Markup, name: &str) {
        push_attribute(markup, name, self);
    }
}

impl<'a> AttributeValue for std::borrow::Cow<'a, str> {
    fn render_attribute_value(&self, markup: &mut Markup, name: &str) {
        push_attribute(markup, name, self);
    }
}

impl_attribute_value_display!(
    bool, char, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClassList {
    values: Vec<String>,
}

impl ClassList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, value: impl Into<String>) {
        let value = value.into();
        if !value.is_empty() {
            self.values.push(value);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn join(&self) -> String {
        self.values.join(" ")
    }
}

pub trait ClassValue {
    fn push_classes(&self, classes: &mut ClassList);
}

impl<T> ClassValue for Option<T>
where
    T: ClassValue,
{
    fn push_classes(&self, classes: &mut ClassList) {
        if let Some(value) = self {
            value.push_classes(classes);
        }
    }
}

impl ClassValue for String {
    fn push_classes(&self, classes: &mut ClassList) {
        classes.push(self.clone());
    }
}

impl ClassValue for str {
    fn push_classes(&self, classes: &mut ClassList) {
        classes.push(self);
    }
}

impl ClassValue for &str {
    fn push_classes(&self, classes: &mut ClassList) {
        classes.push(*self);
    }
}

impl<'a> ClassValue for std::borrow::Cow<'a, str> {
    fn push_classes(&self, classes: &mut ClassList) {
        classes.push(self.as_ref());
    }
}

impl<T, const N: usize> ClassValue for [T; N]
where
    T: ClassValue,
{
    fn push_classes(&self, classes: &mut ClassList) {
        for value in self {
            value.push_classes(classes);
        }
    }
}

impl<T> ClassValue for [T]
where
    T: ClassValue,
{
    fn push_classes(&self, classes: &mut ClassList) {
        for value in self {
            value.push_classes(classes);
        }
    }
}

impl<T> ClassValue for Vec<T>
where
    T: ClassValue,
{
    fn push_classes(&self, classes: &mut ClassList) {
        for value in self {
            value.push_classes(classes);
        }
    }
}

macro_rules! impl_class_value_tuple {
    ($($name:ident),+ $(,)?) => {
        impl<$($name),+> ClassValue for ($($name,)+)
        where
            $($name: ClassValue),+
        {
            fn push_classes(&self, classes: &mut ClassList) {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                $(
                    $name.push_classes(classes);
                )+
            }
        }
    };
}

impl_class_value_tuple!(A, B);
impl_class_value_tuple!(A, B, C);
impl_class_value_tuple!(A, B, C, D);

#[doc(hidden)]
pub mod __private {
    use super::{AttributeValue, ClassList, ClassValue, Markup, RenderValue};

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
        super::push_attribute(markup, name, value);
    }

    pub fn push_attribute_expr<T>(markup: &mut Markup, name: &str, value: &T)
    where
        T: AttributeValue + ?Sized,
    {
        value.render_attribute_value(markup, name);
    }

    pub fn push_boolean_attribute(markup: &mut Markup, name: &str) {
        markup.push_raw(" ");
        markup.push_raw(name);
    }

    pub fn push_class_value<T>(classes: &mut ClassList, value: &T)
    where
        T: ClassValue + ?Sized,
    {
        value.push_classes(classes);
    }

    pub fn push_class_attribute(markup: &mut Markup, classes: &ClassList) {
        if !classes.is_empty() {
            super::push_attribute(markup, "class", &classes.join());
        }
    }

    pub fn finish_open_tag(markup: &mut Markup) {
        markup.push_raw(">");
    }

    pub fn end_element(markup: &mut Markup, name: &str) {
        markup.push_raw("</");
        markup.push_raw(name);
        markup.push_raw(">");
    }

    pub fn is_void_element(name: &str) -> bool {
        matches!(
            name,
            "area"
                | "base"
                | "br"
                | "col"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
    }
}

fn push_attribute(markup: &mut Markup, name: &str, value: &(impl Display + ?Sized)) {
    markup.push_raw(" ");
    markup.push_raw(name);
    markup.push_raw("=\"");
    markup.push_text(value.to_string());
    markup.push_raw("\"");
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
    use super::{Markup, document, html, page};

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

    #[test]
    fn supports_empty_tags_and_shorthand_selectors() {
        let markup = html! {
            section #hero .page .stack {
                meta charset="utf-8";
                input #email .field type="email";
            }
        };

        assert_eq!(
            markup.as_str(),
            "<section id=\"hero\" class=\"page stack\"><meta charset=\"utf-8\"><input id=\"email\" class=\"field\" type=\"email\"></section>"
        );
    }

    #[test]
    fn omits_optional_attribute_values() {
        let label: Option<&str> = None;

        let markup = html! {
            button type="button" aria_label=(label) { "Save" }
        };

        assert_eq!(markup.as_str(), "<button type=\"button\">Save</button>");
    }

    #[test]
    fn renders_document_builder_for_layouts() {
        fn app_layout(title: &str, content: Markup) -> Markup {
            page(title)
                .lang("en")
                .body_class("app-shell")
                .head(html! {
                    meta name="description" content="SSW demo";
                    link rel="stylesheet" href="/app.css";
                })
                .body(html! {
                    main #app .page {
                        header {
                            h1 { (title) }
                        }
                        (content)
                    }
                })
                .render()
        }

        let page = app_layout(
            "Dashboard",
            html! {
                section .panel {
                    p { "Everything is rendered on the server." }
                }
            },
        );

        assert!(
            page.as_str()
                .starts_with("<!DOCTYPE html><html lang=\"en\">")
        );
        assert!(page.as_str().contains("<body class=\"app-shell\">"));
        assert!(
            page.as_str()
                .contains("<link rel=\"stylesheet\" href=\"/app.css\">")
        );
        assert!(page.as_str().contains("<main id=\"app\" class=\"page\">"));
        assert!(page.as_str().contains(
            "<section class=\"panel\"><p>Everything is rendered on the server.</p></section>"
        ));
    }

    #[test]
    fn merges_shorthand_and_explicit_class_values() {
        let extra = Some("featured");

        let markup = html! {
            article .card class=(("stack", extra)) {
                "Hello"
            }
        };

        assert_eq!(
            markup.as_str(),
            "<article class=\"card stack featured\">Hello</article>"
        );
    }

    #[test]
    fn omits_empty_class_attribute_when_all_values_are_missing() {
        let none_class: Option<&str> = None;

        let markup = html! {
            div class=(none_class) {
                "Empty"
            }
        };

        assert_eq!(markup.as_str(), "<div>Empty</div>");
    }
}
