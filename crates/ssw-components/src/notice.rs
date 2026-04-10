use ssw_core::{FlashLevel, FlashMessage};
use ssw_html::{Markup, html};

fn notice_role(level: FlashLevel) -> &'static str {
    match level {
        FlashLevel::Info | FlashLevel::Success => "status",
        FlashLevel::Warning | FlashLevel::Error => "alert",
    }
}

/// Renders a simple informational notice component.
pub fn alert(message: impl AsRef<str>) -> Markup {
    html! {
        div class=(("ssw-notice", "ssw-notice--info")) data_level="info" role=(notice_role(FlashLevel::Info)) {
            p class="ssw-notice__message" {
                (message.as_ref())
            }
        }
    }
}

/// Renders a flash-style notice using semantic level classes.
pub fn flash_notice(flash: &FlashMessage) -> Markup {
    let level = flash.level().as_str();
    let level_class = format!("ssw-notice--{level}");
    let role = notice_role(flash.level());

    html! {
        div class=(("ssw-notice", level_class.as_str())) data_level=(level) role=(role) {
            p class="ssw-notice__message" { (flash.message()) }
        }
    }
}
