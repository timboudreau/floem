use super::{window_handle::WindowHandle, window_handle_utils::WindowHandleNative};
use crate::style::CursorStyle;
use adapters::{WindowResizeDirection, WindowSystemTheme};
use peniko::kurbo::{Point, Size, Vec2};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windowing::public_api::NativeWindow;
use winit::{
    cursor::CursorIcon,
    dpi::{LogicalPosition, LogicalSize},
    window::{ImeCapabilities, ImeEnableRequest, ImeHint, ImePurpose, ImeRequest, ImeRequestData},
};

impl WindowHandleNative for WindowHandle {
    fn set_cursor(&mut self) {
        let cursor = match self.window_state.cursor {
            Some(CursorStyle::Default) => CursorIcon::Default,
            Some(CursorStyle::Pointer) => CursorIcon::Pointer,
            Some(CursorStyle::Progress) => CursorIcon::Progress,
            Some(CursorStyle::Wait) => CursorIcon::Wait,
            Some(CursorStyle::Crosshair) => CursorIcon::Crosshair,
            Some(CursorStyle::Text) => CursorIcon::Text,
            Some(CursorStyle::Move) => CursorIcon::Move,
            Some(CursorStyle::Grab) => CursorIcon::Grab,
            Some(CursorStyle::Grabbing) => CursorIcon::Grabbing,
            Some(CursorStyle::ColResize) => CursorIcon::ColResize,
            Some(CursorStyle::RowResize) => CursorIcon::RowResize,
            Some(CursorStyle::WResize) => CursorIcon::WResize,
            Some(CursorStyle::EResize) => CursorIcon::EResize,
            Some(CursorStyle::NwResize) => CursorIcon::NwResize,
            Some(CursorStyle::NeResize) => CursorIcon::NeResize,
            Some(CursorStyle::SwResize) => CursorIcon::SwResize,
            Some(CursorStyle::SeResize) => CursorIcon::SeResize,
            Some(CursorStyle::SResize) => CursorIcon::SResize,
            Some(CursorStyle::NResize) => CursorIcon::NResize,
            Some(CursorStyle::NeswResize) => CursorIcon::NeswResize,
            Some(CursorStyle::NwseResize) => CursorIcon::NwseResize,
            None => CursorIcon::Default,
        };
        if cursor != self.window_state.last_cursor {
            // Baseview's set cursor impl is `todo!()`
            #[cfg(all(feature = "winit", not(feature = "baseview")))]
            self.window.set_cursor(cursor.into());
            self.window_state.last_cursor = cursor;
        }
    }

    fn init_renderer_wasm(&mut self) {
        // On the web, we need to get the canvas size once. The size will be updated automatically
        // when the canvas element is resized subsequently. This is the correct place to do so
        // because the renderer is not initialized until now.
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWeb;

            let rect = self
                .window
                .as_ref()
                .unwrap()
                .canvas()
                .unwrap()
                .get_bounding_client_rect();
            // let rect = canvas.get_bounding_client_rect();
            let size = LogicalSize::new(rect.width(), rect.height());
            self.size(Size::new(size.width, size.height));
        }
    }

    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window
            .window_handle()
            .expect("Window should have a handle")
            .as_raw()
    }

    fn set_window_theme(&self, theme: Option<WindowSystemTheme>) {
        self.window.set_theme(theme.map(WindowSystemTheme::into))
    }

    fn window_pre_present_notify(window: &NativeWindow) {
        window.pre_present_notify();
    }

    fn schedule_repaint(&self) {
        self.window.request_redraw();
    }

    fn set_window_visible(&mut self, visible: bool) {
        self.window.set_visible(visible);
    }

    fn set_ime_allowed(&mut self, allowed: bool) {
        if self.window.ime_capabilities().is_some() != allowed {
            let ime = if allowed {
                let position = LogicalPosition::new(0, 0);
                let size = LogicalSize::new(0, 0);
                let request_data = ImeRequestData::default()
                    .with_cursor_area(position.into(), size.into())
                    .with_hint_and_purpose(ImeHint::NONE, ImePurpose::Normal);

                ImeRequest::Enable(
                    ImeEnableRequest::new(
                        ImeCapabilities::new()
                            .with_hint_and_purpose()
                            .with_cursor_area(),
                        request_data,
                    )
                    .unwrap(),
                )
            } else {
                ImeRequest::Disable
            };
            self.window.request_ime_update(ime).unwrap();
        }
    }

    fn set_ime_cursor_area(&mut self, position: Point, size: Size) {
        if self
            .window
            .ime_capabilities()
            .map(|caps| caps.cursor_area())
            .unwrap_or(false)
        {
            let position = winit::dpi::Position::Logical(winit::dpi::LogicalPosition::new(
                position.x * self.window_state.scale,
                position.y * self.window_state.scale,
            ));
            let size = winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                size.width * self.window_state.scale,
                size.height * self.window_state.scale,
            ));
            self.window
                .request_ime_update(ImeRequest::Update(
                    ImeRequestData::default().with_cursor_area(position, size),
                ))
                .unwrap();
        }
    }

    fn set_window_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    fn focus_window(&mut self) {
        self.window.focus_window();
    }

    fn drag_window(&mut self) {
        let _ = self.window.drag_window();
    }

    fn drag_resize_window(&mut self, direction: Option<WindowResizeDirection>) {
        // If this message came from a native windowing event, and the windowing system
        // is baseview, there is no concept of window resize directions there.
        if let Some(direction) = direction {
            let _ = self.window.drag_resize_window(direction.into());
        } else {
            let _ = self
                .window
                .drag_resize_window(WindowResizeDirection::default().into());
        }
    }

    fn is_window_maximized(&self) -> bool {
        self.window.is_maximized()
    }

    fn set_window_maximized(&mut self, maximized: bool) {
        self.window.set_maximized(maximized);
    }

    fn set_window_minimized(&mut self, minimized: bool) {
        self.window.set_minimized(minimized);
    }

    fn set_window_position(&mut self, delta: Vec2) {
        let pos = self.window_position + delta;
        self.window
            .set_outer_position(winit::dpi::Position::Logical(
                winit::dpi::LogicalPosition::new(pos.x, pos.y),
            ));
    }
}
