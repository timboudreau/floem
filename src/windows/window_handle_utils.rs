use adapters::{WindowResizeDirection, WindowSystemTheme};
use peniko::kurbo::{Point, Size, Vec2};
use raw_window_handle::RawWindowHandle;
use windowing::public_api::NativeWindow;

/// While we don't strictly need an abstraction for this, since there will only be one implementation
/// (but a different one based on a feature flag) and we could implement these methods directly on
/// `WindowHandle`'s implementation, having a trait for this imposes some discipline that makes it less
/// likely that anyone will break one windowing framework's implementation and not know it, since this
/// trait forms a contract for what it must implement.
pub(crate) trait WindowHandleNative {
     fn set_cursor(&mut self);
     fn init_renderer_wasm(&mut self);
     fn raw_window_handle(&self) -> RawWindowHandle;
     fn set_window_theme(&self, theme : Option<WindowSystemTheme>);
     fn window_pre_present_notify(window: &NativeWindow);
     fn schedule_repaint(&self);
     fn set_window_visible(&mut self, visible : bool);
     fn set_ime_allowed(&mut self, allowed : bool);
     fn set_ime_cursor_area(&mut self, position: Point, size: Size);
     fn set_window_title(&mut self, title : &str);
     fn focus_window(&mut self);
     fn drag_window(&mut self);
     fn drag_resize_window(&mut self, direction : Option<WindowResizeDirection>);
     fn is_window_maximized(&self) -> bool;
     fn set_window_maximized(&mut self, maximized : bool);
     fn set_window_minimized(&mut self, minimized : bool);
     fn set_window_position(&mut self, delta : Vec2);
}
