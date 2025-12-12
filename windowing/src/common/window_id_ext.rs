use super::ScreenLayout;
#[cfg(feature = "winit")]
use crate::common::Urgency;
#[cfg(target_os = "macos")]
use crate::internal_api::WindowUpdate;
use peniko::kurbo::{Point, Rect, Size};

/// Ensures `WindowIdExt` cannot be implemented on arbitrary types.
pub(crate) trait WindowIdExtSealed: Sized + Copy {
    fn add_window_update(&self, msg: WindowUpdate);
}

/// Extends `WindowId` to give instances methods to retrieve properties of the associated window,
/// much as `ViewId` does.
///
/// Methods may return None if the view is not realized on-screen, or
/// if information needed to compute the result is not available on the current platform or
/// available on the current platform but not from the calling thread.
///
/// **Platform support notes:**
///  * macOS: Many of the methods here, if called from a thread other than `main`, are
///    blocking because accessing most window properties may only be done from the main
///    thread on that OS.
///  * Android & Wayland: Getting the outer position of a window is not supported by `winit` and
///    methods whose return value have that as a prerequisite will return `None` or return a
///    reasonable default.
///  * X11: Some window managers (Openbox was one such which was tested) *appear* to support
///    retrieving separate window-with-frame and window-content positions and sizes, but in
///    fact report the same values for both.
#[allow(private_bounds)]
pub trait WindowIdExt: WindowIdExtSealed {
    /// Get the bounds of the content of this window, including
    /// titlebar and native window borders.
    fn bounds_on_screen_including_frame(&self) -> Option<Rect>;
    /// Get the bounds of the content of this window, excluding
    /// titlebar and native window borders.
    fn bounds_of_content_on_screen(&self) -> Option<Rect>;
    /// Get the location of the window including any OS titlebar.
    fn position_on_screen_including_frame(&self) -> Option<Point>;
    /// Get the location of the window's content on the monitor where
    /// it currently resides, **excluding** any OS titlebar.
    fn position_of_content_on_screen(&self) -> Option<Point>;
    /// Get the logical bounds of the monitor this window is on.
    fn monitor_bounds(&self) -> Option<Rect>;
    /// Determine if this window is currently visible.  Note that if a
    /// call to set a window visible which is invisible has happened within
    /// the current event loop cycle, the state returned will not reflect that.
    fn is_visible(&self) -> bool;
    /// Determine if this window is currently minimized. Note that if a
    /// call to minimize or unminimize this window, and it is currently in the
    /// opposite state, has happened the current event loop cycle, the state
    /// returned will not reflect that.
    fn is_minimized(&self) -> bool;

    /// Determine if this window is currently maximize. Note that if a
    /// call to maximize or unmaximize this window, and it is currently in the
    /// opposite state, has happened the current event loop cycle, the state
    /// returned will not reflect that.
    fn is_maximized(&self) -> bool;

    /// Determine if the window decorations should indicate an edited, unsaved
    /// document.  Platform-dependent: Will only ever return `true` on macOS.
    fn is_document_edited(&self) -> bool;

    /// Instruct the window manager to indicate in the window's decorations
    /// that the window contains an unsaved, edited document.  Only has an
    /// effect on macOS.
    #[allow(unused_variables)] // edited unused on non-mac builds
    fn set_document_edited(&self, edited: bool) {
        #[cfg(target_os = "macos")]
        self.add_window_update(WindowUpdate::DocumentEdited(edited))
    }

    /// Set this window's visible state, hiding or showing it if it has been
    /// hidden
    fn set_visible(&self, visible: bool) {
        self.add_window_update(WindowUpdate::Visibility(visible))
    }

    /// Update the bounds of this window.
    fn set_window_inner_bounds(&self, bounds: Rect) {
        self.add_window_update(WindowUpdate::InnerBounds(bounds))
    }

    /// Update the bounds of this window.
    fn set_window_outer_bounds(&self, bounds: Rect) {
        self.add_window_update(WindowUpdate::OuterBounds(bounds))
    }

    /// Change this window's maximized state.
    fn maximized(&self, maximized: bool) {
        self.add_window_update(WindowUpdate::Maximize(maximized))
    }

    /// Change this window's minimized state.
    fn minimized(&self, minimized: bool) {
        self.add_window_update(WindowUpdate::Minimize(minimized))
    }

    /// Change this window's minimized state.
    fn set_outer_location(&self, location: Point) {
        self.add_window_update(WindowUpdate::OuterLocation(location))
    }

    /// Ask the OS's windowing framework to update the size of the window
    /// based on the passed size for its *content* (excluding titlebar, frame
    /// or other decorations).
    fn set_content_size(&self, size: Size) {
        self.add_window_update(WindowUpdate::InnerSize(size))
    }

    #[cfg(feature = "winit")]
    /// Cause the desktop to perform some attention-drawing behavior that draws
    /// the user's attention specifically to this window - e.g. bouncing in
    /// the dock on macOS.  On X11, after calling this method with some urgency
    /// other than `None`, it is necessary to *clear* the attention-seeking state
    /// by calling this method again with `Urgency::None`.
    fn request_attention(&self, urgency: impl Into<Urgency>) {
        // use Impl<Into> to allow old code using winit types to compile
        self.add_window_update(WindowUpdate::RequestAttention(urgency.into()))
    }

    /// Force a repaint of this window through the native window's repaint mechanism,
    /// bypassing floem's normal repaint mechanism.
    ///
    /// This method may be removed or deprecated in the future, but has been needed
    /// in [some situations](https://github.com/lapce/floem/issues/463), and to
    /// address a few ongoing issues in `winit` (window unmaximize is delayed until
    /// an external event triggers a repaint of the requesting window), and may
    /// be needed as a workaround if other such issues are discovered until they
    /// can be addressed.
    ///
    /// Returns true if the repaint request was issued successfully (i.e. there is
    /// an actual system-level window corresponding to this `WindowId`).
    fn force_repaint(&self) -> bool;

    /// Get a layout of this window in relation to the monitor on which it currently
    /// resides, if any.
    fn screen_layout(&self) -> Option<ScreenLayout>;

    /// Get the dots-per-inch scaling of this window or 1.0 if the platform does not
    /// support it (Android).
    fn scale(&self) -> f64;
}
