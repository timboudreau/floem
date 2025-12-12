use adapters::WindowResizeDirection;
use crate::{
    action::drag_resize_window,
    event::EventListener,
    ViewId,
    ViewIdentifier,
    style::CursorStyle,
    view::{IntoView, View},
};

use super::Decorators;

/// A view that will resize the window when the mouse is dragged. See [`drag_resize_window_area`].
///
/// ## Platform-specific
///
/// - **macOS:** Not supported.
/// - **iOS / Android / Web / Orbital:** Not supported.
pub struct DragResizeWindowArea {
    id: ViewId,
}

/// A view that will resize the window when the mouse is dragged.
///
/// ## Platform-specific
///
/// - **macOS:** Not supported.
/// - **iOS / Android / Web / Orbital:** Not supported.
pub fn drag_resize_window_area<V: IntoView + 'static>(
    direction: WindowResizeDirection,
    child: V,
) -> DragResizeWindowArea {
    let id = ViewId::new();
    id.set_children([child.into_view()]);
    DragResizeWindowArea { id }
        .on_event_stop(EventListener::PointerDown, move |_| {
            drag_resize_window(direction)
        })
        .style(move |s| {
            let cursor = match direction {
                WindowResizeDirection::East => CursorStyle::ColResize,
                WindowResizeDirection::West => CursorStyle::ColResize,
                WindowResizeDirection::North => CursorStyle::RowResize,
                WindowResizeDirection::South => CursorStyle::RowResize,
                WindowResizeDirection::NorthEast => CursorStyle::NeswResize,
                WindowResizeDirection::SouthWest => CursorStyle::NeswResize,
                WindowResizeDirection::SouthEast => CursorStyle::NwseResize,
                WindowResizeDirection::NorthWest => CursorStyle::NwseResize,
            };
            s.cursor(cursor)
        })
}

impl View for DragResizeWindowArea {
    fn id(&self) -> ViewId {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "Drag-Resize Window Area".into()
    }
}
