use winit::dpi::{LogicalPosition, LogicalSize};

use crate::{
    NativeWindow, ScreenLayout, ViewId, WindowIdentifier,
    internal_api::{WindowUpdate, WindowingBackend, WindowingBackendInternal},
    private::window_tracking::with_window,
    winit::{
        window_geometry::bounds_to_logical_outer_position_and_inner_size,
        winit_screen_layout::try_create_screen_layout,
    },
};
use std::{cell::RefCell, collections::HashMap};

// Using thread_local for consistency with static vars in updates.rs, but I suspect these
// are thread_local not because thread-locality is desired, but only because static mutability is
// desired - but that's a patch for another day.
thread_local! {
    /// Holding pen for window state changes, processed as part of the event loop cycle
    pub(crate) static WINDOW_UPDATE_MESSAGES: RefCell<HashMap<WindowIdentifier, Vec<WindowUpdate>>> = Default::default();
}

#[derive(Copy, Clone, Debug)]
pub enum WInit {}

impl WindowingBackend for WInit {
    fn retreive_window_updates(id: &WindowIdentifier) -> Option<Vec<WindowUpdate>> {
        WINDOW_UPDATE_MESSAGES.with_borrow_mut(|map| map.remove(id))
    }

    fn screen_layout_of(view: &ViewId) -> Option<ScreenLayout> {
        try_create_screen_layout(view)
    }

    fn process_window_updates(id: &WindowIdentifier) -> bool {
        let mut result = false;
        if let Some(items) = crate::WindowingSystem::retreive_window_updates(id) {
            result = !items.is_empty();
            for update in items {
                match update {
                    WindowUpdate::Visibility(visible) => {
                        with_window(id, |window| {
                            window.set_visible(visible);
                        });
                    }
                    #[allow(unused_variables)] // non mac - edited is unused
                    WindowUpdate::DocumentEdited(edited) => {
                        #[cfg(target_os = "macos")]
                        with_window(id, |window| {
                            use winit::platform::macos::WindowExtMacOS;
                            window.set_document_edited(edited);
                        });
                    }
                    WindowUpdate::OuterBounds(bds) => {
                        with_window(id, |window| {
                            let params =
                                bounds_to_logical_outer_position_and_inner_size(window, bds, true);
                            window.set_outer_position(params.0.into());
                            // XXX log any returned error?
                            let _ = window.request_surface_size(params.1.into());
                        });
                    }
                    WindowUpdate::InnerBounds(bds) => {
                        with_window(id, |window| {
                            let params =
                                bounds_to_logical_outer_position_and_inner_size(window, bds, false);
                            window.set_outer_position(params.0.into());
                            // XXX log any returned error?
                            let _ = window.request_surface_size(params.1.into());
                        });
                    }
                    WindowUpdate::RequestAttention(att) => {
                        with_window(id, |window| {
                            window.request_user_attention(att);
                        });
                    }
                    WindowUpdate::Minimize(minimize) => {
                        with_window(id, |window| {
                            window.set_minimized(minimize);
                            if !minimize {
                                // If we don't trigger a repaint on macOS,
                                // unminimize doesn't happen until an input
                                // event arrives. Unrelated to
                                // https://github.com/lapce/floem/issues/463 -
                                // this is in winit or below.
                                maybe_yield_with_repaint(window);
                            }
                        });
                    }
                    WindowUpdate::Maximize(maximize) => {
                        with_window(id, |window| window.set_maximized(maximize));
                    }
                    WindowUpdate::OuterLocation(outer) => {
                        with_window(id, |window| {
                            window
                                .set_outer_position(LogicalPosition::new(outer.x, outer.y).into());
                        });
                    }
                    WindowUpdate::InnerSize(size) => {
                        with_window(id, |window| {
                            window.request_surface_size(
                                LogicalSize::new(size.width, size.height).into(),
                            )
                        });
                    }
                }
            }
        }
        result
    }
}

impl WindowingBackendInternal for WInit {
    fn push_window_update(id: &WindowIdentifier, msg: WindowUpdate) {
        WINDOW_UPDATE_MESSAGES.with_borrow_mut(|map| match map.entry(*id) {
            std::collections::hash_map::Entry::Occupied(updates) => {
                updates.into_mut().push(msg);
            }
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(vec![msg]);
            }
        });
    }
}

/// Some operations - notably minimize and restoring visibility - don't take
/// effect on macOS until something triggers a repaint in the target window - the
/// issue is below the level of floem's event loops and seems to be in winit or
/// deeper.  Workaround is to force the window to repaint.
#[allow(unused_variables)] // non mac builds see `window` as unused
fn maybe_yield_with_repaint(window: &NativeWindow) {
    #[cfg(target_os = "macos")]
    {
        window.request_redraw();
        let main = Some("main") != std::thread::current().name();
        if !main {
            // attempt to get out of the way of the main thread
            std::thread::yield_now();
        }
    }
}
