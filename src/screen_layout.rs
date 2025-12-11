//! Tools for computing screen locations from locations within a View and
//! vice-versa.

use crate::{ViewId, ViewIdentifier};
use peniko::kurbo::Point;
use windowing::internal_api::{WindowingBackend, WindowingSystem};

pub(crate) fn ensure_windowing_system_initialized() {
    // Sanity check that the window system type, which varies by backend, is zero-sized -
    // it should not be stateful in any way, shape or form.
    const { debug_assert!(std::mem::size_of::<WindowingSystem>() == 0) };
    WindowingSystem::init(find_root, find_window_origin);
}

fn find_root(view : &ViewId) -> Option<ViewId> {
    view.root()
}

fn find_window_origin(view: &ViewId) -> Point {
    let mut pt = Point::ZERO;
    recursively_find_window_origin(*view, &mut pt);
    pt
}

fn recursively_find_window_origin(view: ViewId, point: &mut Point) {
    if let Some(layout) = view.get_layout() {
        point.x += layout.location.x as f64;
        point.y += layout.location.y as f64;
        if let Some(parent) = view.parent() {
            recursively_find_window_origin(parent, point);
        }
    }
}
