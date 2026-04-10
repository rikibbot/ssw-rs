use ssw_html::{Markup, page as html_page};

/// Renders a full page using the `ssw-html` document builder.
pub fn page(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    html_page(title.as_ref()).body(body).render()
}
