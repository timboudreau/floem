use super::WindowUpdate;
use crate::{NativeWindow, ScreenLayout, ViewId, WindowIdentifier};
use peniko::kurbo::Point;
use std::sync::OnceLock;

pub type RootFinder = fn(&ViewId) -> Option<ViewId>;
pub type OriginFinder = fn(view: &ViewId) -> Point;

static ROOT_FINDER: OnceLock<(RootFinder, OriginFinder)> = OnceLock::new();

/// The windowing backend - currently `winit` or `baseview` - depending on which feature flag
/// is selected, there will be a type named `WindowingSystem` which implements support for one or the
/// other.
///
/// All functionality is implemented as associated functions, and implementations should be zero-sized.
pub trait WindowingBackend: Sized {
    /// Because, of necessity, we define `ViewId` in this crate, but not the entire panoply of functionality
    /// available through it, at application start we must have a few functions that can call implementation
    /// methods set up - this allows us to have a single implementation of a few caches which are dependent
    /// on being able to look up the root view from a view id, and find the origin of a view's outermost ancestor.
    fn init(root_finder: RootFinder, origin_finder: OriginFinder) {
        let _ = ROOT_FINDER.set((root_finder, origin_finder));
    }

    /// Retrieve any pending window updates for processing.
    fn retreive_window_updates(id: &WindowIdentifier) -> Option<Vec<WindowUpdate>>;

    /// Called by `ApplicationHandle` at the end of the event loop callback to process window updates.
    fn process_window_updates(id: &WindowIdentifier) -> bool;

    /// Given a `ViewId`, locate its outermost parent.
    fn root_of(view_id: &ViewId) -> Option<ViewId> {
        let f = ROOT_FINDER
            .get()
            .expect("Windowing backend not initialized")
            .0;
        f(view_id)
    }

    /// Given a `ViewId`, get the origin of its outermost ancestor within the window that owns it (if unowned,
    /// returns a `0,0` location).  For views in windows that have decorations, the returned value is likely not
    /// to be `0,0` but to take into account the window's decorations (this may vary by platform and windowing
    /// back end; it is true for Mac OS).
    fn origin_of(view_id: &ViewId) -> Point {
        let f = ROOT_FINDER
            .get()
            .expect("Windowing backend not initialized")
            .1;
        f(view_id)
    }

    /// Get a screen layout, given a view, if possible, that describes that view's relative to the geometry of
    /// the monitor it appears on.  This is needed to position popup windows optimally.
    ///
    /// The default implementation returns `None` but it is implemented for the `winit` backend.
    fn screen_layout_of(_view: &ViewId) -> Option<ScreenLayout> {
        None
    }

    /// Determine if the passed view id is currently registered as the outermost view in any window.
    fn is_known_root(view_id: &ViewId) -> bool {
        crate::private::window_tracking::is_known_root(view_id)
    }

    /// Get the id of the window, if any, that contains the view identified by the passed id, *if and only if*
    /// it is the outermost view within that window.
    fn window_id_for_root(root_id: ViewId) -> Option<WindowIdentifier> {
        crate::private::window_tracking::window_id_for_root(root_id)
    }

    /// Record a root view, the window it is in and the native window the identifier represents.
    fn store_window_id_mapping(
        root_id: ViewId,
        window_id: WindowIdentifier,
        window: &NativeWindow,
    ) {
        crate::private::window_tracking::store_window_id_mapping(root_id, window_id, window);
    }

    /// Remove the mapping for view and its window.
    fn remove_window_id_mapping(root_id: &ViewId, window_id: &WindowIdentifier) {
        crate::private::window_tracking::remove_window_id_mapping(root_id, window_id);
    }

    /// Call the passed function with the native window for a given window id, if there is one.
    fn with_window<F: FnOnce(&NativeWindow) -> T, T>(window: &WindowIdentifier, f: F) -> Option<T> {
        crate::private::window_tracking::with_window(window, f)
    }

    /// Get the root view of a window if there is one.
    fn root_view_in(window: &WindowIdentifier) -> Option<ViewId> {
        crate::private::window_tracking::root_view_id(window)
    }
}

pub(crate) trait WindowingBackendInternal {
    /// Push a window update to be processed at the end of the event processing cycle.
    fn push_window_update(id: &WindowIdentifier, msg: WindowUpdate);
}
