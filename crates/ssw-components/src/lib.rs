//! Optional UI components built on top of `ssw-html`.

mod button;
mod data;
mod field;
mod layout;
mod notice;
mod page;

pub use button::{button, button_with_variant, link_button, submit_button, ButtonVariant};
pub use data::{
    badge, badge_with_variant, data_table, stat_list, BadgeVariant, StatItem, TableCell, TableRow,
};
pub use field::{
    email_input, field, hidden_input, select, text_input, textarea, Field, SelectOption,
};
pub use layout::{container, section, stack};
pub use notice::{alert, flash_notice, validation_summary, ValidationItem};
pub use page::{
    breadcrumbs, card_header, empty_state, meta_list, page_actions, page_header, page_shell,
    pagination, top_nav, BreadcrumbItem, MetaItem, NavItem, PaginationItem,
};

#[cfg(test)]
mod tests {
    use ssw_core::FlashMessage;
    use ssw_html::Markup;

    use super::{
        alert, badge, badge_with_variant, breadcrumbs, button, button_with_variant, card_header,
        container, data_table, email_input, empty_state, flash_notice, hidden_input, link_button,
        meta_list, page_actions, page_header, page_shell, pagination, section, select, stack,
        stat_list, submit_button, text_input, textarea, top_nav, validation_summary, BadgeVariant,
        BreadcrumbItem, ButtonVariant, Field, MetaItem, NavItem, PaginationItem, SelectOption,
        StatItem, TableCell, TableRow, ValidationItem,
    };

    #[test]
    fn alert_escapes_message() {
        let markup = alert("<unsafe>");

        assert!(markup.as_str().contains("&lt;unsafe&gt;"));
    }

    #[test]
    fn text_input_renders_error_state() {
        let field = Field::new("email", "email", "Email")
            .value("sprite-at-example.com")
            .error(Some("Email must look valid."))
            .required(true);

        let markup = email_input(&field);

        assert!(
            markup
                .as_str()
                .contains("<div class=\"ssw-field\" data-invalid=\"true\"><label class=\"ssw-field__label\" for=\"email\">Email</label>")
        );
        assert!(
            markup
                .as_str()
                .contains("class=\"ssw-input\" data-invalid=\"true\" id=\"email\" type=\"email\" name=\"email\" value=\"sprite-at-example.com\"")
        );
        assert!(markup.as_str().contains("required"));
        assert!(markup.as_str().contains("aria-invalid=\"true\""));
        assert!(markup.as_str().contains("aria-describedby=\"email-error\""));
        assert!(markup.as_str().contains("<p"));
        assert!(markup.as_str().contains("id=\"email-error\""));
        assert!(markup.as_str().contains("class=\"ssw-field__error\""));
        assert!(markup.as_str().contains("Email must look valid."));
    }

    #[test]
    fn textarea_preserves_value_without_error_markup() {
        let field = Field::new("message", "message", "Message").value("Hello");

        let markup = textarea(&field, 4);

        assert!(
            markup.as_str().contains(
                "<label class=\"ssw-field__label\" for=\"message\">Message</label><textarea class=\"ssw-textarea\" id=\"message\" name=\"message\" rows=\"4\">Hello</textarea>"
            )
        );
        assert!(!markup.as_str().contains("aria-invalid"));
        assert!(!markup.as_str().contains("data-invalid"));
    }

    #[test]
    fn text_input_escapes_submitted_value() {
        let field = Field::new("name", "name", "Name").value("<unsafe>");

        let markup = text_input(&field);

        assert!(markup.as_str().contains("value=\"&lt;unsafe&gt;\""));
    }

    #[test]
    fn flash_notice_uses_semantic_level_classes() {
        let markup = flash_notice(&FlashMessage::success("Saved"));

        assert!(markup.as_str().contains(
            "class=\"ssw-notice ssw-notice--success\" data-level=\"success\" role=\"status\""
        ));
        assert!(markup.as_str().contains("class=\"ssw-notice__message\""));
        assert!(markup.as_str().contains("Saved"));
    }

    #[test]
    fn error_flash_notice_uses_alert_role() {
        let markup = flash_notice(&FlashMessage::error("Failed"));

        assert!(markup
            .as_str()
            .contains("data-level=\"error\" role=\"alert\""));
    }

    #[test]
    fn validation_summary_renders_message_and_linked_items() {
        let items = [
            ValidationItem::link("#name", "Name is required."),
            ValidationItem::link("#email", "Email must look valid."),
        ];
        let markup = validation_summary("Please fix the highlighted fields.", &items);

        assert!(markup
            .as_str()
            .contains("class=\"ssw-notice ssw-notice--error ssw-validation-summary\""));
        assert!(markup
            .as_str()
            .contains("Please fix the highlighted fields."));
        assert!(markup
            .as_str()
            .contains("<ul class=\"ssw-validation-summary__list\">"));
        assert!(markup.as_str().contains(
            "<a class=\"ssw-validation-summary__link\" href=\"#name\">Name is required.</a>"
        ));
        assert!(markup.as_str().contains(
            "<a class=\"ssw-validation-summary__link\" href=\"#email\">Email must look valid.</a>"
        ));
    }

    #[test]
    fn validation_summary_supports_unlinked_items() {
        let items = [ValidationItem::new(
            "Your form expired. Reload the page and try again.",
        )];
        let markup = validation_summary("Submission failed.", &items);

        assert!(markup.as_str().contains("Submission failed."));
        assert!(markup.as_str().contains("<span class=\"ssw-validation-summary__text\">Your form expired. Reload the page and try again.</span>"));
    }

    #[test]
    fn hidden_input_renders_hidden_control() {
        let markup = hidden_input("csrf_token", "abc123");

        assert!(markup
            .as_str()
            .contains("type=\"hidden\" name=\"csrf_token\" value=\"abc123\""));
    }

    #[test]
    fn button_uses_default_primary_variant() {
        let markup = button("Save");

        assert!(markup.as_str().contains(
            "<button class=\"ssw-button\" data-variant=\"primary\" type=\"button\">Save</button>"
        ));
    }

    #[test]
    fn button_supports_secondary_variant() {
        let markup = button_with_variant("Cancel", ButtonVariant::Secondary);

        assert!(markup.as_str().contains("data-variant=\"secondary\""));
        assert!(markup.as_str().contains("Cancel"));
    }

    #[test]
    fn submit_button_sets_submit_type() {
        let markup = submit_button("Send");

        assert!(markup.as_str().contains("type=\"submit\""));
        assert!(markup.as_str().contains("Send"));
    }

    #[test]
    fn link_button_renders_stable_link_markup() {
        let markup = link_button("/projects", "Browse projects");

        assert!(markup
            .as_str()
            .contains("<a class=\"ssw-link-button\" href=\"/projects\">Browse projects</a>"));
    }

    #[test]
    fn layout_primitives_wrap_content_with_stable_classes() {
        let markup = container(section(stack(Markup::text("Hello"))));

        assert!(markup.as_str().contains("<div class=\"ssw-container\">"));
        assert!(markup.as_str().contains("<section class=\"ssw-section\">"));
        assert!(markup
            .as_str()
            .contains("<div class=\"ssw-stack\">Hello</div>"));
    }

    #[test]
    fn layout_primitives_escape_plain_text_content() {
        let markup = container("<unsafe>");

        assert!(markup.as_str().contains("&lt;unsafe&gt;"));
        assert!(!markup.as_str().contains("<unsafe>"));
    }

    #[test]
    fn select_marks_current_option_and_uses_stable_classes() {
        let field = Field::new("topic", "topic", "Topic")
            .value("support")
            .required(true);
        let options = [
            SelectOption::new("", "Choose a topic"),
            SelectOption::new("support", "Support"),
            SelectOption::new("sales", "Sales"),
        ];

        let markup = select(&field, &options);

        assert!(markup.as_str().contains("class=\"ssw-select\""));
        assert!(markup
            .as_str()
            .contains("<option value=\"support\" selected>Support</option>"));
        assert!(markup
            .as_str()
            .contains("<label class=\"ssw-field__label\" for=\"topic\">Topic</label>"));
    }

    #[test]
    fn page_shell_primitives_render_stable_structure() {
        let markup = page_shell(page_header(
            "Server Side Web",
            "Rendered on the server.",
            Markup::text("A quieter, reusable page shell."),
            Some(page_actions(Markup::text("Actions"))),
        ));

        assert!(markup.as_str().contains("<div class=\"ssw-page-shell\">"));
        assert!(markup
            .as_str()
            .contains("<header class=\"ssw-page-header\">"));
        assert!(markup
            .as_str()
            .contains("<p class=\"ssw-page-header__eyebrow\">Server Side Web</p>"));
        assert!(markup
            .as_str()
            .contains("<h1 class=\"ssw-page-header__title\">Rendered on the server.</h1>"));
        assert!(markup
            .as_str()
            .contains("<div class=\"ssw-page-actions\">Actions</div>"));
    }

    #[test]
    fn card_header_escapes_plain_text_body() {
        let markup = card_header("Overview", "<unsafe>");

        assert!(markup
            .as_str()
            .contains("<h2 class=\"ssw-card-header__title\">Overview</h2>"));
        assert!(markup.as_str().contains("&lt;unsafe&gt;"));
        assert!(!markup.as_str().contains("<unsafe>"));
    }

    #[test]
    fn top_nav_marks_the_current_item() {
        let items = [
            NavItem::new("/projects", "Projects").current(true),
            NavItem::new("/archive", "Archive"),
        ];
        let markup = top_nav("/", "Server Side Web", &items);

        assert!(markup.as_str().contains("class=\"ssw-top-nav\""));
        assert!(markup
            .as_str()
            .contains("class=\"ssw-top-nav__brand\" href=\"/\">Server Side Web</a>"));
        assert!(markup.as_str().contains(
            "href=\"/projects\" data-current=\"true\" aria-current=\"page\">Projects</a>"
        ));
        assert!(markup.as_str().contains("href=\"/archive\">Archive</a>"));
    }

    #[test]
    fn breadcrumbs_render_links_and_current_page_state() {
        let items = [
            BreadcrumbItem::link("/projects", "Projects"),
            BreadcrumbItem::current("Northstar launch sprint"),
        ];
        let markup = breadcrumbs(&items);

        assert!(markup.as_str().contains("class=\"ssw-breadcrumbs\""));
        assert!(markup
            .as_str()
            .contains("<a class=\"ssw-breadcrumbs__link\" href=\"/projects\">Projects</a>"));
        assert!(markup.as_str().contains(
            "<span class=\"ssw-breadcrumbs__current\" aria-current=\"page\">Northstar launch sprint</span>"
        ));
    }

    #[test]
    fn empty_state_renders_optional_actions() {
        let markup = empty_state(
            "No projects",
            Markup::text("Start by adding one."),
            Some(page_actions(Markup::text("Create"))),
        );

        assert!(markup.as_str().contains("class=\"ssw-empty-state\""));
        assert!(markup
            .as_str()
            .contains("<h2 class=\"ssw-empty-state__title\">No projects</h2>"));
        assert!(markup.as_str().contains("Start by adding one."));
        assert!(markup
            .as_str()
            .contains("class=\"ssw-empty-state__actions\""));
    }

    #[test]
    fn meta_list_renders_labeled_rows() {
        let items = [
            MetaItem::new("Status", "Active"),
            MetaItem::new("Owner", "Mina"),
        ];
        let markup = meta_list(&items);

        assert!(markup.as_str().contains("class=\"ssw-meta-list\""));
        assert!(
            markup
                .as_str()
                .contains("<dt class=\"ssw-meta-list__label\">Status</dt><dd class=\"ssw-meta-list__value\">Active</dd>")
        );
        assert!(
            markup
                .as_str()
                .contains("<dt class=\"ssw-meta-list__label\">Owner</dt><dd class=\"ssw-meta-list__value\">Mina</dd>")
        );
    }

    #[test]
    fn badge_renders_variant_hook_and_escapes_plain_text() {
        let markup = badge_with_variant("<active>", BadgeVariant::Success);

        assert!(markup
            .as_str()
            .contains("<span class=\"ssw-badge\" data-variant=\"success\">&lt;active&gt;</span>"));
    }

    #[test]
    fn badge_defaults_to_neutral_variant() {
        let markup = badge("Queued");

        assert!(markup
            .as_str()
            .contains("<span class=\"ssw-badge\" data-variant=\"neutral\">Queued</span>"));
    }

    #[test]
    fn data_table_renders_column_headers_and_row_headers() {
        let rows = [TableRow::new(vec![
            TableCell::row_header("Northstar"),
            TableCell::new("Active"),
            TableCell::new("May 28"),
        ])];
        let markup = data_table(&["Project", "Status", "Due"], &rows);

        assert!(markup.as_str().contains("class=\"ssw-table-wrapper\""));
        assert!(markup
            .as_str()
            .contains("<th class=\"ssw-table__heading\" scope=\"col\">Project</th>"));
        assert!(markup.as_str().contains(
            "<th class=\"ssw-table__cell ssw-table__cell--row-header\" scope=\"row\">Northstar</th>"
        ));
        assert!(markup
            .as_str()
            .contains("<td class=\"ssw-table__cell\">Active</td>"));
    }

    #[test]
    fn pagination_renders_links_and_current_page_state() {
        let items = [
            PaginationItem::link("/projects?page=1", "Previous"),
            PaginationItem::current("1"),
            PaginationItem::link("/projects?page=2", "2"),
            PaginationItem::link("/projects?page=2", "Next"),
        ];
        let markup = pagination(&items);

        assert!(markup.as_str().contains("class=\"ssw-pagination\""));
        assert!(markup
            .as_str()
            .contains("<a class=\"ssw-pagination__link\" href=\"/projects?page=1\">Previous</a>"));
        assert!(markup
            .as_str()
            .contains("<span class=\"ssw-pagination__current\" aria-current=\"page\">1</span>"));
    }

    #[test]
    fn stat_list_renders_items_with_optional_detail() {
        let items = [
            StatItem::new(
                "Status",
                badge_with_variant("Active", BadgeVariant::Success),
            ),
            StatItem::new("Owner", "Mina").detail("Primary contact"),
        ];
        let markup = stat_list(&items);

        assert!(markup.as_str().contains("class=\"ssw-stat-list\""));
        assert!(markup
            .as_str()
            .contains("<dt class=\"ssw-stat-list__label\">Status</dt>"));
        assert!(
            markup
                .as_str()
                .contains("<dd class=\"ssw-stat-list__value\"><span class=\"ssw-badge\" data-variant=\"success\">Active</span></dd>")
        );
        assert!(markup
            .as_str()
            .contains("<div class=\"ssw-stat-list__detail\">Primary contact</div>"));
    }
}
