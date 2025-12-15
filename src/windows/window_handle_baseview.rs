use super::{window_handle::WindowHandle, window_handle_utils::WindowHandleNative};
use crate::application::baseview_hacks::current_window;
use adapters::{WindowResizeDirection, WindowSystemTheme};
use peniko::kurbo::{Point, Size, Vec2};
use raw_window_handle::{RawWindowHandle};
use windowing::public_api::NativeWindow;

// Basically, almost none of this is implementable over baseview windows.
#[allow(dead_code, unused)]
impl WindowHandleNative for WindowHandle {
    fn set_cursor(&mut self) {}

    fn init_renderer_wasm(&mut self) {}

    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.window
    }

    fn set_window_theme(&self, _theme: Option<WindowSystemTheme>) {}

    fn window_pre_present_notify(_window: &NativeWindow) {}

    fn schedule_repaint(&self) {}

    fn set_window_visible(&mut self, _visible: bool) {}

    fn set_ime_allowed(&mut self, _allowed: bool) {}

    fn set_ime_cursor_area(&mut self, _position: Point, _size: Size) {}

    fn set_window_title(&mut self, _title: &str) {}

    fn focus_window(&mut self) {
        current_window().focus();
    }

    fn drag_window(&mut self) {}

    fn drag_resize_window(&mut self, _direction: Option<WindowResizeDirection>) {}

    fn is_window_maximized(&self) -> bool {
        false
    }

    fn set_window_maximized(&mut self, _maximized: bool) {}

    fn set_window_minimized(&mut self, _minimized: bool) {}

    fn set_window_position(&mut self, _delta: Vec2) {}
}
