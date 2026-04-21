//! Scoped CSS authoring primitives for `ssw-rs`.
//!
//! This crate provides a small experimental `css!` macro for component-local
//! styles. It emits normal CSS, keeps scoping deterministic, and does not rely
//! on any client-side runtime.

extern crate self as ssw_css;

use ssw_html::Markup;

pub use ssw_css_macros::css;

/// A rendered scoped stylesheet with deterministic local class names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleSheet {
    scope: &'static str,
    css: &'static str,
    slots: &'static [&'static str],
}

impl StyleSheet {
    /// Creates a stylesheet from pre-rendered scoped CSS.
    #[doc(hidden)]
    pub const fn new(
        scope: &'static str,
        css: &'static str,
        slots: &'static [&'static str],
    ) -> Self {
        Self { scope, css, slots }
    }

    /// Returns the deterministic scope identifier for this stylesheet.
    pub fn scope(&self) -> &'static str {
        self.scope
    }

    /// Returns the rendered scoped CSS text.
    pub fn render(&self) -> &'static str {
        self.css
    }

    /// Returns whether the stylesheet defines a local slot with the given name.
    pub fn has_slot(&self, slot: &str) -> bool {
        self.slots.contains(&slot)
    }

    /// Returns the fully scoped class name for a local slot, if it exists.
    pub fn try_class(&self, slot: &str) -> Option<String> {
        self.has_slot(slot)
            .then(|| format!("sswc-{}-{slot}", self.scope))
    }

    /// Returns the fully scoped class name for a local slot.
    ///
    /// Panics when the slot name is not present in the stylesheet.
    pub fn class(&self, slot: &str) -> String {
        self.try_class(slot)
            .unwrap_or_else(|| panic!("unknown scoped css slot `{slot}`"))
    }

    /// Returns a space-separated class string for multiple local slots.
    ///
    /// Panics when any slot name is not present in the stylesheet.
    pub fn classes<I, S>(&self, slots: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        slots
            .into_iter()
            .map(|slot| self.class(slot.as_ref()))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Renders the stylesheet as an inline `<style>` tag.
    pub fn style_tag(&self) -> Markup {
        let mut markup = Markup::raw("<style data-ssw-css=\"");
        markup.push_raw(self.scope);
        markup.push_raw("\">");
        markup.push_raw(self.css);
        markup.push_raw("</style>");
        markup
    }
}

impl From<&StyleSheet> for Markup {
    fn from(styles: &StyleSheet) -> Self {
        styles.style_tag()
    }
}

impl From<StyleSheet> for Markup {
    fn from(styles: StyleSheet) -> Self {
        styles.style_tag()
    }
}

#[cfg(test)]
mod tests {
    use ssw_html::html;

    use super::css;

    #[test]
    fn renders_scoped_css_and_resolves_slot_classes() {
        let styles = css! {
            ".root" {
                display: grid;
                gap: 1 rem;
            }

            ".title" {
                font-weight: 600;
            }

            ".root:hover .title" {
                color: var(--accent);
            }
        };

        let root = styles.class("root");
        let title = styles.class("title");

        assert!(root.starts_with("sswc-"));
        assert!(root.ends_with("-root"));
        assert!(title.ends_with("-title"));
        assert!(
            styles
                .render()
                .contains(&format!(".{root}{{display:grid;gap:1rem;}}"))
        );
        assert!(
            styles
                .render()
                .contains(&format!(".{root}:hover .{title}{{color:var(--accent);}}"))
        );
    }

    #[test]
    fn supports_media_queries() {
        let styles = css! {
            ".root" {
                gap: 1 rem;
            }

            @media (min-width: 48 rem) {
                ".root" {
                    gap: 1.5 rem;
                }
            }
        };

        let root = styles.class("root");

        assert!(styles.render().contains(&format!(".{root}{{gap:1rem;}}")));
        assert!(styles.render().contains(&format!(
            "@media (min-width:48rem){{.{root}{{gap:1.5rem;}}}}"
        )));
    }

    #[test]
    fn style_tag_integrates_with_html_rendering() {
        let styles = css! {
            ".root" {
                padding: 1 rem;
            }
        };

        let markup = html! {
            (styles.style_tag())
            div class=(styles.class("root")) { "Scoped" }
        };

        assert!(markup.as_str().contains("<style data-ssw-css=\""));
        assert!(markup.as_str().contains("Scoped"));
        assert!(markup.as_str().contains("class=\""));
    }

    #[test]
    fn classes_joins_multiple_local_slots() {
        let styles = css! {
            ".root" {
                display: block;
            }

            ".active" {
                color: red;
            }
        };

        let classes = styles.classes(["root", "active"]);

        assert!(classes.contains("sswc-"));
        assert!(classes.contains("-root"));
        assert!(classes.contains("-active"));
        assert!(classes.contains(' '));
    }

    #[test]
    fn unknown_slots_are_rejected() {
        let styles = css! {
            ".root" {
                display: "block";
            }
        };

        assert!(styles.try_class("missing").is_none());
    }
}
