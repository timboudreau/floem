use adapters::{WindowResizeDirection, WindowSystemTheme};
use peniko::kurbo::{Point, Size, Vec2};
use raw_window_handle::RawWindowHandle;
use windowing::public_api::NativeWindow;
use super::{window_handle::WindowHandle, window_handle_utils::WindowHandleNative};

impl WindowHandleNative for WindowHandle {
    fn set_cursor(&mut self) {

    }

    fn init_renderer_wasm(&mut self) {

    }

    fn raw_window_handle(&self) -> RawWindowHandle {
        todo!()
    }

    fn set_window_theme(&self, theme : Option<WindowSystemTheme>) {

    }

    fn window_pre_present_notify(window: &NativeWindow) {

    }

    fn schedule_repaint(&self) {

    }

    fn set_window_visible(&mut self, visible : bool) {

    }

    fn set_ime_allowed(&mut self, allowed : bool) {

    }

    fn set_ime_cursor_area(&mut self, position: Point, size: Size) {

    }

    fn set_window_title(&mut self, title : &str) {

    }

    fn focus_window(&mut self) {

    }

    fn drag_window(&mut self) {

    }

    fn drag_resize_window(&mut self, direction : Option<WindowResizeDirection>) {

    }

    fn is_window_maximized(&self) -> bool {
        false
    }

    fn set_window_maximized(&mut self, maximized : bool) {

    }

    fn set_window_minimized(&mut self, minimized : bool) {

    }

    fn set_window_position(&mut self, delta : Vec2) {

    }
}
