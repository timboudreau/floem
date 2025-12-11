use crate::{
    public_api::{NativeWindow, ScreenLayout, ViewId, WindowIdentifier},
    internal_api::{WindowingBackend as _, WindowingSystem},
    private::window_tracking::with_window_id_and_window,
    winit::window_geometry::{monitor_bounds_for_monitor, rect_from_physical_bounds_for_window},
};

/// Create a `ScreenLayout` for a view.  This can fail if the view or an
/// ancestor of it has no parent and is not realized on-screen, or if the
/// platform does not support reading window inner or outer bounds.  `ScreenLayout`
/// is useful when needing to convert locations within a view into absolute
/// positions on-screen, such as for creating a window at position relative
/// to that view.
pub(crate) fn try_create_screen_layout(view: &ViewId) -> Option<ScreenLayout> {
    with_window_id_and_window(view, |window_id, window| {
        window.current_monitor().and_then(|monitor| {
            let inner_position = window.surface_position();
            window
                .outer_position()
                .map(|outer_position| {
                    let monitor_bounds = monitor_bounds_for_monitor(window, &monitor);
                    let inner_size = window.surface_size();
                    let outer_size = window.outer_size();

                    let window_bounds =
                        rect_from_physical_bounds_for_window(window, outer_position, outer_size);

                    let window_content_bounds =
                        rect_from_physical_bounds_for_window(window, inner_position, inner_size);

                    let view_origin_in_window = WindowingSystem::origin_of(view);
                    let monitor_scale = window.scale_factor();

                    ScreenLayout {
                        monitor_scale,
                        monitor_bounds,
                        window_content_bounds,
                        window_bounds,
                        view_origin_in_window: Some(view_origin_in_window),
                        window_id: *window_id,
                    }
                })
                .ok()
        })
    })
    .unwrap_or(None)
}

pub fn screen_layout_for_window(
    window_id: WindowIdentifier,
    window: &NativeWindow,
) -> Option<ScreenLayout> {
    window.current_monitor().and_then(|monitor| {
        let inner_position = window.surface_position();
        window
            .outer_position()
            .map(|outer_position| {
                let monitor_bounds = monitor_bounds_for_monitor(window, &monitor);
                let inner_size = window.surface_size();
                let outer_size = window.outer_size();

                let window_bounds =
                    rect_from_physical_bounds_for_window(window, outer_position, outer_size);

                let window_content_bounds =
                    rect_from_physical_bounds_for_window(window, inner_position, inner_size);

                let view_origin_in_window = None;
                let monitor_scale = window.scale_factor();

                ScreenLayout {
                    monitor_scale,
                    monitor_bounds,
                    window_content_bounds,
                    window_bounds,
                    view_origin_in_window,
                    window_id,
                }
            })
            .ok()
    })
}
