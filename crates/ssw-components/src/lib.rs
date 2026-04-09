//! Optional UI components built on top of `ssw-html`.

use ssw_html::{Markup, html, page as html_page};

/// Renders a simple status alert component.
pub fn alert(message: impl AsRef<str>) -> Markup {
    html! {
        div .ssw_alert role="status" {
            (message.as_ref())
        }
    }
}

/// Renders a full page using the `ssw-html` document builder.
pub fn page(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    html_page(title.as_ref()).body(body).render()
}

#[cfg(test)]
mod tests {
    use super::alert;

    #[test]
    fn alert_escapes_message() {
        let markup = alert("<unsafe>");

        assert!(markup.as_str().contains("&lt;unsafe&gt;"));
    }
}
