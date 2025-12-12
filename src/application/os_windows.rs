//! New window customizations for Windows.
use winit::platform::windows::WindowAttributesWindows;

pub(super) fn apply_windows_specialization(
    mut window_attributes: winit::window::WindowAttributes,
    undecorated_shadow: bool,
    config: &Option<WinOSWindowConfig>,
) -> winit::window::WindowAttributes {
    let mut win = WindowAttributesWindows::default().with_undecorated_shadow(undecorated_shadow);
    if let Some(cfg) = config {
        use crate::window::convert_to_win;
        win = win
            .with_title_background_color(convert_to_win(cfg.set_title_background_color))
            .with_border_color(convert_to_win(cfg.set_border_color))
            .with_skip_taskbar(cfg.set_skip_taskbar)
            .with_corner_preference(cfg.corner_preference.into())
            .with_system_backdrop(cfg.set_system_backdrop.into())
            .with_title_text_color(convert_to_win(cfg.set_title_text_color).unwrap_or_default());
    }
    window_attributes.with_platform_attributes(Box::new(win))
}
