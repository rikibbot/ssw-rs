//! HTML authoring primitives for `ssw-rs`.
//!
//! This crate exposes the public HTML surface of the framework:
//! [`Markup`], [`Document`], convenience page builders, and the [`html!`] macro.

extern crate self as ssw_html;

use std::fmt::{Display, Write};

use ssw_core::{HtmlKind, HtmlResponse, Render};

pub use ssw_html_macros::html;

/// Font loading helpers for document `<head>` markup.
pub mod fonts {
    use super::{Markup, html};

    /// The supported `font-display` strategies for Google Fonts stylesheets.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FontDisplay {
        /// Lets the browser choose the loading strategy.
        Auto,
        /// Hides text until the font is ready.
        Block,
        /// Uses fallback text immediately, then swaps in the font.
        Swap,
        /// Uses a short block period before falling back.
        Fallback,
        /// Uses the font only if it loads immediately.
        Optional,
    }

    impl FontDisplay {
        fn as_str(self) -> &'static str {
            match self {
                Self::Auto => "auto",
                Self::Block => "block",
                Self::Swap => "swap",
                Self::Fallback => "fallback",
                Self::Optional => "optional",
            }
        }
    }

    /// The supported font file formats for local `@font-face` rules.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FontFormat {
        /// A modern compressed web font.
        Woff2,
        /// The older Web Open Font Format.
        Woff,
        /// A TrueType font file.
        TrueType,
        /// An OpenType font file.
        OpenType,
    }

    impl FontFormat {
        fn css_format(self) -> &'static str {
            match self {
                Self::Woff2 => "woff2",
                Self::Woff => "woff",
                Self::TrueType => "truetype",
                Self::OpenType => "opentype",
            }
        }

        fn mime_type(self) -> &'static str {
            match self {
                Self::Woff2 => "font/woff2",
                Self::Woff => "font/woff",
                Self::TrueType => "font/ttf",
                Self::OpenType => "font/otf",
            }
        }
    }

    /// The supported `font-style` values for local `@font-face` rules.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FontStyle {
        /// Upright text.
        Normal,
        /// Italic text.
        Italic,
        /// Oblique text.
        Oblique,
    }

    impl FontStyle {
        fn as_str(self) -> &'static str {
            match self {
                Self::Normal => "normal",
                Self::Italic => "italic",
                Self::Oblique => "oblique",
            }
        }
    }

    /// A small helper for rendering Google Fonts `<link>` tags.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GoogleFont {
        family: String,
        weights: Vec<u16>,
        display: FontDisplay,
    }

    impl GoogleFont {
        /// Creates a Google Fonts helper for a font family name.
        pub fn new(family: impl Into<String>) -> Self {
            Self {
                family: family.into(),
                weights: Vec::new(),
                display: FontDisplay::Swap,
            }
        }

        /// Adds a single font weight to the stylesheet request.
        pub fn weight(mut self, weight: u16) -> Self {
            self.weights.push(weight);
            self
        }

        /// Replaces the requested font weights.
        pub fn weights(mut self, weights: &[u16]) -> Self {
            self.weights = weights.to_vec();
            self
        }

        /// Sets the `font-display` strategy for the stylesheet request.
        pub fn display(mut self, display: FontDisplay) -> Self {
            self.display = display;
            self
        }

        /// Uses `font-display: swap`.
        pub fn display_swap(self) -> Self {
            self.display(FontDisplay::Swap)
        }

        /// Returns the Google Fonts stylesheet URL.
        pub fn stylesheet_url(&self) -> String {
            let family = self.family.trim().replace(' ', "+");
            let weights = normalized_weights(&self.weights);

            if weights.is_empty() {
                return format!(
                    "https://fonts.googleapis.com/css2?family={family}&display={}",
                    self.display.as_str()
                );
            }

            let weights = weights
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(";");

            format!(
                "https://fonts.googleapis.com/css2?family={family}:wght@{weights}&display={}",
                self.display.as_str()
            )
        }

        /// Renders preconnect and stylesheet tags for the font.
        pub fn render(&self) -> Markup {
            let stylesheet_url = self.stylesheet_url();

            html! {
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link rel="stylesheet" href=(stylesheet_url);
            }
        }
    }

    impl From<GoogleFont> for Markup {
        fn from(font: GoogleFont) -> Self {
            font.render()
        }
    }

    impl From<&GoogleFont> for Markup {
        fn from(font: &GoogleFont) -> Self {
            font.render()
        }
    }

    /// A small helper for rendering a local `@font-face` rule and optional preload tag.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LocalFontFace {
        family: String,
        source: String,
        format: Option<FontFormat>,
        display: FontDisplay,
        style: FontStyle,
        weight: Option<String>,
        preload: bool,
        cross_origin: bool,
    }

    impl LocalFontFace {
        /// Creates a local font helper for a family name and file URL.
        pub fn new(family: impl Into<String>, source: impl Into<String>) -> Self {
            Self {
                family: family.into(),
                source: source.into(),
                format: None,
                display: FontDisplay::Swap,
                style: FontStyle::Normal,
                weight: None,
                preload: false,
                cross_origin: false,
            }
        }

        /// Declares the file format for the font source.
        pub fn format(mut self, format: FontFormat) -> Self {
            self.format = Some(format);
            self
        }

        /// Sets the `font-display` strategy for the font face.
        pub fn display(mut self, display: FontDisplay) -> Self {
            self.display = display;
            self
        }

        /// Uses `font-display: swap`.
        pub fn display_swap(self) -> Self {
            self.display(FontDisplay::Swap)
        }

        /// Sets the `font-style` for the font face.
        pub fn style(mut self, style: FontStyle) -> Self {
            self.style = style;
            self
        }

        /// Uses `font-style: normal`.
        pub fn normal(self) -> Self {
            self.style(FontStyle::Normal)
        }

        /// Uses `font-style: italic`.
        pub fn italic(self) -> Self {
            self.style(FontStyle::Italic)
        }

        /// Uses `font-style: oblique`.
        pub fn oblique(self) -> Self {
            self.style(FontStyle::Oblique)
        }

        /// Sets a single font weight.
        pub fn weight(mut self, weight: u16) -> Self {
            self.weight = Some(weight.to_string());
            self
        }

        /// Sets an inclusive font-weight range for variable fonts.
        pub fn weight_range(mut self, start: u16, end: u16) -> Self {
            self.weight = Some(format!("{start} {end}"));
            self
        }

        /// Sets a custom `font-weight` value.
        pub fn weight_value(mut self, value: impl Into<String>) -> Self {
            self.weight = Some(value.into());
            self
        }

        /// Emits a preload link for the font source.
        pub fn preload(mut self) -> Self {
            self.preload = true;
            self
        }

        /// Adds `crossorigin` to the preload link.
        pub fn cross_origin(mut self) -> Self {
            self.cross_origin = true;
            self
        }

        /// Renders preload and `@font-face` markup for the font.
        pub fn render(&self) -> Markup {
            let mut markup = Markup::new();

            if self.preload {
                let mime_type = self.format.map(FontFormat::mime_type);
                let crossorigin = if self.cross_origin {
                    Some("anonymous")
                } else {
                    None
                };

                markup.append(html! {
                    link
                        rel="preload"
                        href=(self.source.as_str())
                        as="font"
                        type=(mime_type)
                        crossorigin=(crossorigin);
                });
            }

            let mut css = String::from("@font-face{font-family:\"");
            push_css_string(&mut css, &self.family);
            css.push_str("\";src:url(\"");
            push_css_string(&mut css, &self.source);
            css.push_str("\")");

            if let Some(format) = self.format {
                css.push_str(" format(\"");
                css.push_str(format.css_format());
                css.push_str("\")");
            }

            css.push(';');
            css.push_str("font-display:");
            css.push_str(self.display.as_str());
            css.push(';');
            css.push_str("font-style:");
            css.push_str(self.style.as_str());
            css.push(';');

            if let Some(weight) = &self.weight {
                css.push_str("font-weight:");
                css.push_str(weight);
                css.push(';');
            }

            css.push('}');

            markup.push_raw("<style>");
            markup.push_raw(&css);
            markup.push_raw("</style>");
            markup
        }
    }

    impl From<LocalFontFace> for Markup {
        fn from(font: LocalFontFace) -> Self {
            font.render()
        }
    }

    impl From<&LocalFontFace> for Markup {
        fn from(font: &LocalFontFace) -> Self {
            font.render()
        }
    }

    /// Creates a Google Fonts helper for a family name.
    pub fn google_font(family: impl Into<String>) -> GoogleFont {
        GoogleFont::new(family)
    }

    /// Creates a local `@font-face` helper for a family name and source URL.
    pub fn local_font(family: impl Into<String>, source: impl Into<String>) -> LocalFontFace {
        LocalFontFace::new(family, source)
    }

    fn normalized_weights(weights: &[u16]) -> Vec<u16> {
        let mut weights = weights.to_vec();
        weights.sort_unstable();
        weights.dedup();
        weights
    }

    fn push_css_string(output: &mut String, value: &str) {
        for ch in value.chars() {
            match ch {
                '\\' => output.push_str("\\\\"),
                '"' => output.push_str("\\\""),
                '\n' => output.push_str("\\a "),
                '\r' => {}
                '<' => output.push_str("\\3C "),
                '>' => output.push_str("\\3E "),
                '&' => output.push_str("\\26 "),
                _ => output.push(ch),
            }
        }
    }
}

/// An owned HTML buffer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Markup(String);

impl Markup {
    /// Creates an empty markup buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates markup from trusted raw HTML.
    pub fn raw(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }

    /// Escapes text and stores it as markup.
    pub fn text(value: impl AsRef<str>) -> Self {
        let mut markup = Self::new();
        markup.push_text(value);
        markup
    }

    /// Appends trusted raw HTML to the buffer.
    pub fn push_raw(&mut self, value: impl AsRef<str>) {
        self.0.push_str(value.as_ref());
    }

    /// Escapes and appends text to the buffer.
    pub fn push_text(&mut self, value: impl AsRef<str>) {
        escape_into(&mut self.0, value.as_ref());
    }

    /// Appends another markup value.
    pub fn append(&mut self, markup: impl Into<Markup>) {
        self.0.push_str(&markup.into().0);
    }

    /// Consumes the markup and returns the owned HTML string.
    pub fn into_string(self) -> String {
        self.0
    }

    /// Borrows the rendered HTML string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts this markup into an `ssw-core` HTML response.
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

/// A minimal document builder for full HTML pages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    title: String,
    lang: String,
    head: Markup,
    body: Markup,
    body_class: Option<String>,
}

impl Document {
    /// Creates a document builder with a page title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lang: "en".to_owned(),
            head: Markup::new(),
            body: Markup::new(),
            body_class: None,
        }
    }

    /// Sets the root `<html lang="...">` value.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// Appends markup to the document `<head>`.
    pub fn head(mut self, markup: impl Into<Markup>) -> Self {
        self.head.append(markup);
        self
    }

    /// Replaces the document `<body>` content.
    pub fn body(mut self, markup: impl Into<Markup>) -> Self {
        self.body = markup.into();
        self
    }

    /// Sets a class attribute on the document `<body>`.
    pub fn body_class(mut self, class: impl Into<String>) -> Self {
        self.body_class = Some(class.into());
        self
    }

    /// Renders the document into owned markup.
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

/// Starts building a full HTML document.
pub fn page(title: impl Into<String>) -> Document {
    Document::new(title)
}

/// Renders a complete HTML document with a title and body.
pub fn document(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    page(title.as_ref()).body(body).render()
}

/// Wraps a value as an HTML fragment.
pub fn fragment(body: impl Into<Markup>) -> Markup {
    body.into()
}

/// Implementation detail used by `html!` to render expression values.
#[doc(hidden)]
pub trait RenderValue {
    /// Appends the rendered value into the provided markup buffer.
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

/// Implementation detail used by `html!` for attribute rendering.
#[doc(hidden)]
pub trait AttributeValue {
    /// Renders the attribute value into the provided markup buffer.
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
    char, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

impl AttributeValue for bool {
    fn render_attribute_value(&self, markup: &mut Markup, name: &str) {
        if is_boolean_attribute(name) {
            if *self {
                markup.push_raw(" ");
                markup.push_raw(name);
            }
            return;
        }

        push_attribute(markup, name, self);
    }
}

/// Implementation detail used by `html!` for class attribute composition.
#[doc(hidden)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClassList {
    values: Vec<String>,
}

impl ClassList {
    /// Creates an empty class list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a class value when it is not empty.
    pub fn push(&mut self, value: impl Into<String>) {
        let value = value.into();
        if !value.is_empty() {
            self.values.push(value);
        }
    }

    /// Returns whether the list has no classes.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Joins the classes into a single class attribute string.
    pub fn join(&self) -> String {
        self.values.join(" ")
    }
}

/// Implementation detail used by `html!` for class value composition.
#[doc(hidden)]
pub trait ClassValue {
    /// Appends one or more class names into the provided class list.
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
    //! Implementation details used by macro expansion.

    use super::{AttributeValue, ClassList, ClassValue, Markup, RenderValue};

    /// Renders an interpolated expression.
    pub fn render_value<T>(markup: &mut Markup, value: &T)
    where
        T: RenderValue + ?Sized,
    {
        value.render_value(markup);
    }

    /// Starts an opening tag.
    pub fn begin_element(markup: &mut Markup, name: &str) {
        markup.push_raw("<");
        markup.push_raw(name);
    }

    /// Pushes a literal attribute value.
    pub fn push_attribute_literal(markup: &mut Markup, name: &str, value: &str) {
        super::push_attribute(markup, name, value);
    }

    /// Pushes an expression-backed attribute value.
    pub fn push_attribute_expr<T>(markup: &mut Markup, name: &str, value: &T)
    where
        T: AttributeValue + ?Sized,
    {
        value.render_attribute_value(markup, name);
    }

    /// Pushes a boolean attribute by name.
    pub fn push_boolean_attribute(markup: &mut Markup, name: &str) {
        markup.push_raw(" ");
        markup.push_raw(name);
    }

    /// Appends one or more classes from a value into the class list.
    pub fn push_class_value<T>(classes: &mut ClassList, value: &T)
    where
        T: ClassValue + ?Sized,
    {
        value.push_classes(classes);
    }

    /// Writes the `class` attribute when any classes were collected.
    pub fn push_class_attribute(markup: &mut Markup, classes: &ClassList) {
        if !classes.is_empty() {
            super::push_attribute(markup, "class", &classes.join());
        }
    }

    /// Closes an opening tag.
    pub fn finish_open_tag(markup: &mut Markup) {
        markup.push_raw(">");
    }

    /// Writes a closing tag.
    pub fn end_element(markup: &mut Markup, name: &str) {
        markup.push_raw("</");
        markup.push_raw(name);
        markup.push_raw(">");
    }

    /// Returns whether an element is a void HTML element.
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

fn is_boolean_attribute(name: &str) -> bool {
    matches!(
        name,
        "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "default"
            | "defer"
            | "disabled"
            | "formnovalidate"
            | "hidden"
            | "inert"
            | "ismap"
            | "itemscope"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "playsinline"
            | "readonly"
            | "required"
            | "reversed"
            | "selected"
    )
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
    use super::{Markup, document, fonts, html, page};

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

    #[test]
    fn renders_boolean_html_attributes_by_presence() {
        let enabled = true;
        let disabled = false;

        let markup = html! {
            input checked=(enabled) disabled=(disabled);
        };

        assert_eq!(markup.as_str(), "<input checked>");
    }

    #[test]
    fn keeps_true_false_values_for_non_boolean_attributes() {
        let expanded = false;

        let markup = html! {
            button aria_expanded=(expanded) {
                "Toggle"
            }
        };

        assert_eq!(
            markup.as_str(),
            "<button aria-expanded=\"false\">Toggle</button>"
        );
    }

    #[test]
    fn renders_google_font_head_markup() {
        let markup = fonts::google_font("Inter")
            .weights(&[600, 400, 500, 500])
            .display_swap()
            .render();

        assert_eq!(
            markup.as_str(),
            "<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\"><link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin><link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&amp;display=swap\">"
        );
    }

    #[test]
    fn renders_google_font_without_weights() {
        let markup = fonts::google_font("Source Sans 3").render();

        assert_eq!(
            markup.as_str(),
            "<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\"><link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin><link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Source+Sans+3&amp;display=swap\">"
        );
    }

    #[test]
    fn renders_local_font_face_with_preload() {
        let markup = fonts::local_font("Inter", "/static/fonts/Inter.var.woff2")
            .format(fonts::FontFormat::Woff2)
            .weight_range(100, 900)
            .preload()
            .render();

        assert_eq!(
            markup.as_str(),
            "<link rel=\"preload\" href=\"/static/fonts/Inter.var.woff2\" as=\"font\" type=\"font/woff2\"><style>@font-face{font-family:\"Inter\";src:url(\"/static/fonts/Inter.var.woff2\") format(\"woff2\");font-display:swap;font-style:normal;font-weight:100 900;}</style>"
        );
    }

    #[test]
    fn renders_local_font_face_with_custom_style_and_cross_origin() {
        let markup = fonts::local_font(
            "Newsreader",
            "https://cdn.example.com/fonts/newsreader.woff",
        )
        .format(fonts::FontFormat::Woff)
        .italic()
        .weight(600)
        .cross_origin()
        .preload()
        .display(fonts::FontDisplay::Optional)
        .render();

        assert_eq!(
            markup.as_str(),
            "<link rel=\"preload\" href=\"https://cdn.example.com/fonts/newsreader.woff\" as=\"font\" type=\"font/woff\" crossorigin=\"anonymous\"><style>@font-face{font-family:\"Newsreader\";src:url(\"https://cdn.example.com/fonts/newsreader.woff\") format(\"woff\");font-display:optional;font-style:italic;font-weight:600;}</style>"
        );
    }
}
