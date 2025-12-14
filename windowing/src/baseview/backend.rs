use crate::{
    internal_api::{WindowUpdate, WindowingBackend, WindowingBackendInternal},
};
use peniko::kurbo::Size;
use super::window_identifier::WindowIdentifier;

pub enum Baseview {}

impl WindowingBackend for Baseview {
    fn retreive_window_updates(id: &WindowIdentifier) -> Option<Vec<WindowUpdate>> {
        crate::private::window_update_messages::retreive_window_update_messages(id)
    }

    /// Called by `ApplicationHandle` at the end of the event loop callback to process window updates.
    fn process_window_updates(id: &WindowIdentifier) -> bool {
        let mut result = false;
        if let Some(items) = Self::retreive_window_updates(id) {
            result = !items.is_empty();
            for update in items {
                match update {
                    WindowUpdate::Visibility(v) => {
                        println!("update window visibility {v} not implemented");
                    },
                    WindowUpdate::InnerBounds(rect) => {
                        println!("update inner bounds to {:?} not implemented", rect);
                    },
                    WindowUpdate::OuterBounds(rect) => {
                        println!("update outer bounds to {:?} not implemented", rect);
                    },
                    WindowUpdate::OuterLocation(point) => {
                        println!("update outer location to {:?} not implemented", point);
                    },
                    WindowUpdate::InnerSize(size) => {
                        println!("update inner size to {:?} not implemented", size);
                    },
                    WindowUpdate::RequestAttention(urgency) => {
                        println!("Request attention {:?} not implemented", urgency);
                    },
                    WindowUpdate::Minimize(mx) => {
                        println!("Update minimize state to {mx} not implemented");
                    },
                    WindowUpdate::Maximize(mx) => {
                        println!("Update maximize state to {mx} not implemented");
                    },
                    WindowUpdate::DocumentEdited(e) => {
                        println!("Update document edited state to {e} not implemented");
                    },
                }
            }
        }
        result
    }

    fn logical_surface_size(window: &super::NativeWindowInner, _scale: f64) -> Size {
        println!("FIXME: Punting on window surface size for {:?}", window);
        Size::new(1200., 800.)
    }
}

impl WindowingBackendInternal for Baseview {
    fn push_window_update(id: &WindowIdentifier, msg: WindowUpdate) {
        crate::private::window_update_messages::push_window_update_message(id, msg);
    }
}
