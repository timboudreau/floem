use crate::{
    private::window_tracking::with_window_map,
    public_api::{NativeWindow, WindowIdentifier},
};
use peniko::kurbo::{Point, Rect, Size};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Pixel},
    monitor::MonitorHandle,
};

pub fn monitor_bounds(id: &WindowIdentifier) -> Option<Rect> {
    with_window_map(|m| {
        m.with_window(id, |window| {
            window
                .current_monitor()
                .map(|monitor| monitor_bounds_for_monitor(window, &monitor))
        })
        .unwrap_or(None)
    })
    .unwrap_or(None)
}

pub fn monitor_bounds_for_monitor(window: &NativeWindow, monitor: &MonitorHandle) -> Rect {
    let scale = 1.0 / window.scale_factor();
    let pos = monitor.position().unwrap_or_default();
    let sz = monitor
        .current_video_mode()
        .map(|h| h.size())
        .unwrap_or_default();
    let x = pos.x as f64 * scale;
    let y = pos.y as f64 * scale;
    Rect::new(
        x,
        y,
        x + sz.width as f64 * scale,
        y + sz.height as f64 * scale,
    )
}

fn scale_rect(window: &NativeWindow, mut rect: Rect) -> Rect {
    let scale = 1.0 / window.scale_factor();
    rect.x0 *= scale;
    rect.y0 *= scale;
    rect.x1 *= scale;
    rect.y1 *= scale;
    rect
}

fn scale_point(window: &NativeWindow, mut rect: Point) -> Point {
    let scale = 1.0 / window.scale_factor();
    rect.x *= scale;
    rect.y *= scale;
    rect
}

pub fn window_inner_screen_position(id: &WindowIdentifier) -> Option<Point> {
    with_window_map(|m| {
        m.with_window(id, |window| {
            let pos = window.surface_position();
            scale_point(window, Point::new(pos.x as f64, pos.y as f64))
        })
    })
    .unwrap_or(None)
}

pub fn window_inner_screen_bounds(id: &WindowIdentifier) -> Option<Rect> {
    with_window_map(|m| {
        m.with_window(id, |window| {
            let pos = window.surface_position();
            rect_from_physical_bounds_for_window(window, pos, window.surface_size())
        })
    })
    .unwrap_or(None)
}

pub fn rect_from_physical_bounds_for_window(
    window: &NativeWindow,
    pos: PhysicalPosition<i32>,
    sz: PhysicalSize<u32>,
) -> Rect {
    scale_rect(
        window,
        Rect::new(
            pos.x as f64,
            pos.y as f64,
            pos.x as f64 + sz.width as f64,
            pos.y as f64 + sz.height as f64,
        ),
    )
}

pub fn window_outer_screen_position(id: &WindowIdentifier) -> Option<Point> {
    with_window_map(|m| {
        m.with_window(id, |window| {
            window
                .outer_position()
                .map(|pos| Some(scale_point(window, Point::new(pos.x as f64, pos.y as f64))))
                .unwrap_or(None)
        })
        .unwrap_or(None)
    })
    .unwrap_or(None)
}

pub fn window_outer_screen_bounds(id: &WindowIdentifier) -> Option<Rect> {
    with_window_map(|m| {
        m.with_window(id, |window| {
            window
                .outer_position()
                .map(|pos| {
                    Some(rect_from_physical_bounds_for_window(
                        window,
                        pos,
                        window.outer_size(),
                    ))
                })
                .unwrap_or(None)
        })
        .unwrap_or(None)
    })
    .unwrap_or(None)
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
pub(crate) fn bounds_to_logical_outer_position_and_inner_size(
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

pub(crate) fn delta_size(
    inner: PhysicalSize<u32>,
    outer: PhysicalSize<u32>,
    window_scale: f64,
) -> (f64, f64) {
    let inner = winit_phys_size_to_size(inner, window_scale);
    let outer = winit_phys_size_to_size(outer, window_scale);
    (outer.width - inner.width, outer.height - inner.height)
}

type PositionResult = Result<winit::dpi::PhysicalPosition<i32>, winit::error::RequestError>;

pub(crate) fn delta_position(
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

pub(crate) fn winit_position_to_point<I: Into<f64> + Pixel>(pos: LogicalPosition<I>) -> Point {
    Point::new(pos.x.into(), pos.y.into())
}

pub(crate) fn winit_size_to_size<I: Into<f64> + Pixel>(size: LogicalSize<I>) -> Size {
    Size::new(size.width.into(), size.height.into())
}

pub(crate) fn winit_phys_position_to_point<I: Into<f64> + Pixel>(
    pos: PhysicalPosition<I>,
    window_scale: f64,
) -> Point {
    winit_position_to_point::<I>(pos.to_logical(window_scale))
}

pub(crate) fn winit_phys_size_to_size<I: Into<f64> + Pixel>(
    size: PhysicalSize<I>,
    window_scale: f64,
) -> Size {
    winit_size_to_size::<I>(size.to_logical(window_scale))
}
