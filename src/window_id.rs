use crate::{
    ScreenLayout, ViewId, WindowIdExt, WindowIdentifier,
    screen_layout::screen_layout_for_window,
    window_id_ext::{WindowIdExtSealed, WindowUpdate},
    window_tracking::{NativeWindow, force_window_repaint, with_window},
};
use std::{cell::RefCell, collections::HashMap, sync::Arc};

use super::window_tracking::{
    monitor_bounds, root_view_id, window_inner_screen_bounds, window_inner_screen_position,
    window_outer_screen_bounds, window_outer_screen_position,
};
use peniko::kurbo::{Point, Rect, Size};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Pixel},
    window::Window,
};

// Using thread_local for consistency with static vars in updates.rs, but I suspect these
// are thread_local not because thread-locality is desired, but only because static mutability is
// desired - but that's a patch for another day.
thread_local! {
    /// Holding pen for window state changes, processed as part of the event loop cycle
    pub(crate) static WINDOW_UPDATE_MESSAGES: RefCell<HashMap<WindowIdentifier, Vec<WindowUpdate>>> = Default::default();
}

pub(crate) fn retreive_window_updates(id: &WindowIdentifier) -> Option<Vec<WindowUpdate>> {
    WINDOW_UPDATE_MESSAGES.with_borrow_mut(|map| map.remove(id))
}

pub(crate) fn push_window_update(id: &WindowIdentifier, msg: WindowUpdate) {
    WINDOW_UPDATE_MESSAGES.with_borrow_mut(|map| match map.entry(*id) {
        std::collections::hash_map::Entry::Occupied(updates) => {
            updates.into_mut().push(msg);
        }
        std::collections::hash_map::Entry::Vacant(v) => {
            v.insert(vec![msg]);
        }
    });
}

impl WindowIdExtSealed for WindowIdentifier {
    fn add_window_update(&self, msg: WindowUpdate) {
        push_window_update(self, msg);
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

    fn root_view(&self) -> Option<ViewId> {
        root_view_id(self)
    }

    fn screen_layout(&self) -> Option<ScreenLayout> {
        with_window(self, move |window| screen_layout_for_window(*self, window)).unwrap_or(None)
    }

    fn scale(&self) -> f64 {
        with_window(self, |window| window.scale_factor()).unwrap_or(1.0)
    }
}

/// Called by `ApplicationHandle` at the end of the event loop callback.
pub(crate) fn process_window_updates(id: &WindowIdentifier) -> bool {
    let mut result = false;
    if let Some(items) = retreive_window_updates(id) {
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
                        window.set_outer_position(LogicalPosition::new(outer.x, outer.y).into());
                    });
                }
                WindowUpdate::InnerSize(size) => {
                    with_window(id, |window| {
                        window
                            .request_surface_size(LogicalSize::new(size.width, size.height).into())
                    });
                }
            }
        }
    }
    result
}

/// Compute a new logical position and size, given a window, a rectangle and whether the
/// rectangle represents the desired inner or outer bounds of the window.
///
/// This is complex because winit offers us two somewhat contradictory ways of setting
/// the bounds:
///
///  * You can set the **outer** position with `window.set_outer_position(position)`
///  * You can set the **inner** size with `window.request_inner_size(size)`
///  * You can obtain inner and outer sizes and positions, but you can only set outer
///    position and *inner* size
///
/// So we must take the delta of the inner and outer size and/or positions (position
/// availability is more limited by platform), and from that, create an appropriate
/// inner size and outer position based on a `Rect` that represents either inner or
/// outer.
fn bounds_to_logical_outer_position_and_inner_size(
    window: &NativeWindow,
    target_bounds: Rect,
    target_is_outer: bool,
) -> (LogicalPosition<f64>, LogicalSize<f64>) {
    if !window.is_decorated() {
        // For undecorated windows, the inner and outer location and size are always identical
        // so no further work is needed
        return (
            LogicalPosition::new(target_bounds.x0, target_bounds.y0),
            LogicalSize::new(target_bounds.width(), target_bounds.height()),
        );
    }

    let scale = window.scale_factor();
    if target_is_outer {
        // We need to reduce the size we are requesting by the width and height of the
        // OS-added decorations to get the right target INNER size:
        let inner_to_outer_size_delta =
            delta_size(window.surface_size(), window.outer_size(), scale);

        (
            LogicalPosition::new(target_bounds.x0, target_bounds.y0),
            LogicalSize::new(
                (target_bounds.width() + inner_to_outer_size_delta.0).max(0.),
                (target_bounds.height() + inner_to_outer_size_delta.1).max(0.),
            ),
        )
    } else {
        // We need to shift the x/y position we are requesting up and left (negatively)
        // to come up with an *outer* location that makes sense with the passed rectangle's
        // size as an *inner* size
        let size_delta = delta_size(window.surface_size(), window.outer_size(), scale);
        let inner_to_outer_delta: (f64, f64) = if let Some(delta) =
            delta_position(window.surface_position(), window.outer_position(), scale)
        {
            // This is the more accurate way, but may be unavailable on some platforms
            delta
        } else {
            // We have to make a few assumptions here, one of which is that window
            // decorations are horizontally symmetric - the delta-x / 2 equals a position
            // on the perimeter of the window's frame.  A few ancient XWindows window
            // managers (Enlightenment) might violate that assumption, but it is a rarity.
            (
                size_delta.0 / 2.0,
                size_delta.1, // assume vertical is titlebar and give it full weight
            )
        };
        (
            LogicalPosition::new(
                target_bounds.x0 - inner_to_outer_delta.0,
                target_bounds.y0 - inner_to_outer_delta.1,
            ),
            LogicalSize::new(target_bounds.width(), target_bounds.height()),
        )
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

fn delta_size(inner: PhysicalSize<u32>, outer: PhysicalSize<u32>, window_scale: f64) -> (f64, f64) {
    let inner = winit_phys_size_to_size(inner, window_scale);
    let outer = winit_phys_size_to_size(outer, window_scale);
    (outer.width - inner.width, outer.height - inner.height)
}

type PositionResult = Result<winit::dpi::PhysicalPosition<i32>, winit::error::RequestError>;

fn delta_position(
    inner: PhysicalPosition<i32>,
    outer: PositionResult,
    window_scale: f64,
) -> Option<(f64, f64)> {
    if let Ok(outer) = outer {
        let outer = winit_phys_position_to_point(outer, window_scale);
        let inner = winit_phys_position_to_point(inner, window_scale);

        return Some((inner.x - outer.x, inner.y - outer.y));
    }
    None
}

// Conversion functions for winit's size and point types:

fn winit_position_to_point<I: Into<f64> + Pixel>(pos: LogicalPosition<I>) -> Point {
    Point::new(pos.x.into(), pos.y.into())
}

fn winit_size_to_size<I: Into<f64> + Pixel>(size: LogicalSize<I>) -> Size {
    Size::new(size.width.into(), size.height.into())
}

fn winit_phys_position_to_point<I: Into<f64> + Pixel>(
    pos: PhysicalPosition<I>,
    window_scale: f64,
) -> Point {
    winit_position_to_point::<I>(pos.to_logical(window_scale))
}

fn winit_phys_size_to_size<I: Into<f64> + Pixel>(size: PhysicalSize<I>, window_scale: f64) -> Size {
    winit_size_to_size::<I>(size.to_logical(window_scale))
}
