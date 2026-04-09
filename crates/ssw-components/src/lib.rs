use ssw_html::{Markup, document};

pub fn alert(message: impl AsRef<str>) -> Markup {
    let mut markup = Markup::raw("<div class=\"ssw-alert\" role=\"status\">");
    markup.push_text(message);
    markup.push_raw("</div>");
    markup
}

pub fn page(title: impl AsRef<str>, body: impl Into<Markup>) -> Markup {
    document(title, body)
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
