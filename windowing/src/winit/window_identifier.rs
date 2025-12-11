use crate::{
    ScreenLayout, WindowIdExt, WindowIdExtSealed, WindowingSystem,
    internal_api::{WindowUpdate, WindowingBackendInternal},
    private::window_tracking::{force_window_repaint, with_window},
    winit::{
        window_geometry::{
            monitor_bounds, window_inner_screen_bounds, window_inner_screen_position,
            window_outer_screen_bounds, window_outer_screen_position,
        },
        winit_screen_layout::screen_layout_for_window,
    },
};
use peniko::kurbo::{Point, Rect};
use std::ops::{Deref, DerefMut};
use winit::window::WindowId;

/// A transparent wrapper over the library handling window management's window
/// identity abstraction.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct WindowIdentifier {
    id: WindowId,
}

impl crate::common::WindowIdDelegate for WindowIdentifier {}

impl WindowIdentifier {
    pub const fn new(id: WindowId) -> Self {
        Self { id }
    }

    pub const fn into_inner(self) -> WindowId {
        self.id
    }
}

impl From<WindowId> for WindowIdentifier {
    fn from(id: WindowId) -> Self {
        Self::new(id)
    }
}

impl From<WindowIdentifier> for WindowId {
    fn from(value: WindowIdentifier) -> Self {
        value.into_inner()
    }
}

impl From<&WindowId> for WindowIdentifier {
    fn from(id: &WindowId) -> Self {
        Self::new(id.to_owned())
    }
}

impl From<&WindowIdentifier> for WindowId {
    fn from(value: &WindowIdentifier) -> Self {
        value.id.to_owned()
    }
}

impl Deref for WindowIdentifier {
    type Target = WindowId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl DerefMut for WindowIdentifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}

impl WindowIdExtSealed for WindowIdentifier {
    fn add_window_update(&self, msg: WindowUpdate) {
        WindowingSystem::push_window_update(self, msg);
    }
}

impl WindowIdExt for WindowIdentifier {
    fn bounds_on_screen_including_frame(&self) -> Option<Rect> {
        window_outer_screen_bounds(self)
    }

    fn bounds_of_content_on_screen(&self) -> Option<Rect> {
        window_inner_screen_bounds(self)
    }

    fn position_on_screen_including_frame(&self) -> Option<Point> {
        window_outer_screen_position(self)
    }

    fn position_of_content_on_screen(&self) -> Option<Point> {
        window_inner_screen_position(self)
    }

    fn monitor_bounds(&self) -> Option<Rect> {
        monitor_bounds(self)
    }

    fn is_visible(&self) -> bool {
        with_window(self, |window| window.is_visible().unwrap_or(false)).unwrap_or(false)
    }

    fn is_minimized(&self) -> bool {
        with_window(self, |window| window.is_minimized().unwrap_or(false)).unwrap_or(false)
    }

    fn is_maximized(&self) -> bool {
        with_window(self, |window| window.is_maximized()).unwrap_or(false)
    }

    #[cfg(target_os = "macos")]
    #[allow(dead_code)]
    fn is_document_edited(&self) -> bool {
        use winit::platform::macos::WindowExtMacOS;
        with_window(self, |window| window.is_document_edited()).unwrap_or(false)
    }

    #[cfg(not(target_os = "macos"))]
    #[allow(dead_code)]
    fn is_document_edited(&self) -> bool {
        false
    }

    fn force_repaint(&self) -> bool {
        force_window_repaint(self)
    }

    fn screen_layout(&self) -> Option<ScreenLayout> {
        with_window(self, move |window| screen_layout_for_window(*self, window)).unwrap_or(None)
    }

    fn scale(&self) -> f64 {
        with_window(self, |window| window.scale_factor()).unwrap_or(1.0)
    }
}
