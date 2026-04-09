use ssw_core::{HtmlKind, HtmlResponse, Render};

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
    use super::{Markup, document};

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
}
