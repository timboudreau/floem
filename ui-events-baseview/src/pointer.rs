use std::{
    sync::{LazyLock, atomic::AtomicU64},
    time::Instant,
};

use baseview::{MouseButton, MouseEvent};
use dpi::PhysicalPosition;
use keyboard_types::Modifiers;
use ui_events::{
    ScrollDelta,
    pointer::{
        ContactGeometry, PointerButton, PointerButtonEvent, PointerButtons, PointerEvent,
        PointerId, PointerInfo, PointerOrientation, PointerScrollEvent, PointerState, PointerType,
        PointerUpdate,
    },
};

pub fn try_from_baseview_button(b: &MouseButton) -> Option<PointerButton> {
    Some(match b {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Middle => PointerButton::Auxiliary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Back => PointerButton::X1,
        MouseButton::Forward => PointerButton::X2,
        MouseButton::Other(n) => match n {
            6 => PointerButton::B7,
            7 => PointerButton::B8,
            8 => PointerButton::B9,
            9 => PointerButton::B10,
            10 => PointerButton::B11,
            11 => PointerButton::B12,
            12 => PointerButton::B13,
            13 => PointerButton::B14,
            14 => PointerButton::B15,
            15 => PointerButton::B16,
            16 => PointerButton::B17,
            17 => PointerButton::B18,
            18 => PointerButton::B19,
            19 => PointerButton::B20,
            20 => PointerButton::B21,
            21 => PointerButton::B22,
            22 => PointerButton::B23,
            23 => PointerButton::B24,
            24 => PointerButton::B25,
            25 => PointerButton::B26,
            26 => PointerButton::B27,
            27 => PointerButton::B28,
            28 => PointerButton::B29,
            29 => PointerButton::B30,
            30 => PointerButton::B31,
            31 => PointerButton::B32,
            _ => {
                return None;
            }
        },
    })
}

static CACHED_POSITION: LastKnownMousePosition = LastKnownMousePosition::new();

pub fn convert_mouse_event(evt: &MouseEvent, scale_factor: f64) -> Option<PointerEvent> {
    const PRIMARY_MOUSE: PointerInfo = PointerInfo {
        pointer_id: Some(PointerId::PRIMARY),
        persistent_device_id: None,
        pointer_type: PointerType::Mouse,
    };
    Some(match evt {
        MouseEvent::CursorMoved {
            position,
            modifiers,
        } => {
            CACHED_POSITION.update(&position);
            PointerEvent::Move(PointerUpdate {
                pointer: PRIMARY_MOUSE,
                current: pointer_state(
                    *position,
                    keyboard_types::Modifiers::from_bits(modifiers.bits())
                        .expect("Modifiers bits should be compatible"),
                    0,
                    scale_factor,
                ),
                coalesced: vec![],
                predicted: vec![],
            })
        }
        MouseEvent::ButtonPressed { button, modifiers } => PointerEvent::Down(PointerButtonEvent {
            button: try_from_baseview_button(button),
            pointer: PRIMARY_MOUSE,
            state: pointer_state(
                CACHED_POSITION.point(),
                keyboard_types::Modifiers::from_bits(modifiers.bits())
                    .expect("Modifiers bits should be compatible"),
                1,
                scale_factor,
            ),
        }),
        MouseEvent::ButtonReleased { button, modifiers } => PointerEvent::Up(PointerButtonEvent {
            button: try_from_baseview_button(button),
            pointer: PRIMARY_MOUSE,
            state: pointer_state(
                CACHED_POSITION.point(),
                keyboard_types::Modifiers::from_bits(modifiers.bits())
                    .expect("Modifiers bits should be compatible"),
                1,
                scale_factor,
            ),
        }),
        MouseEvent::WheelScrolled { delta, modifiers } => {
            PointerEvent::Scroll(PointerScrollEvent {
                pointer: PRIMARY_MOUSE,
                delta: convert_delta(delta),
                state: pointer_state(
                    CACHED_POSITION.point(),
                    keyboard_types::Modifiers::from_bits(modifiers.bits())
                        .expect("Modifiers bits should be compatible"),
                    1,
                    scale_factor,
                ),
            })
        }
        MouseEvent::CursorEntered => PointerEvent::Enter(PRIMARY_MOUSE),
        MouseEvent::CursorLeft => PointerEvent::Leave(PRIMARY_MOUSE),
        MouseEvent::DragEntered {
            position,
            modifiers: _,
            data: _,
        } => {
            CACHED_POSITION.update(&position);
            // Do something with the clipboard here?
            return None;
        }
        MouseEvent::DragMoved {
            position,
            modifiers: _,
            data: _,
        } => {
            CACHED_POSITION.update(&position);
            return None;
        }
        MouseEvent::DragLeft => return None,
        MouseEvent::DragDropped {
            position,
            modifiers: _,
            data: _,
        } => {
            CACHED_POSITION.update(&position);
            return None;
        }
    })
}

static LAUNCH: LazyLock<Instant> = LazyLock::new(|| Instant::now());

fn convert_delta(delta: &baseview::ScrollDelta) -> ScrollDelta {
    match delta {
        baseview::ScrollDelta::Lines { x, y } => ScrollDelta::LineDelta(*x, *y),
        baseview::ScrollDelta::Pixels { x, y } => ScrollDelta::PixelDelta(PhysicalPosition {
            x: *x as f64,
            y: *y as f64,
        }),
    }
}

fn pointer_state(
    position: baseview::Point,
    modifiers: keyboard_types::Modifiers,
    count: u8,
    scale_factor: f64,
) -> PointerState {
    PointerState {
        time: Instant::now().duration_since(LAUNCH.to_owned()).as_nanos() as u64,
        position: PhysicalPosition {
            x: position.x,
            y: position.y,
        },
        buttons: PointerButtons::new(),
        modifiers: Modifiers::from(modifiers),
        count: count,
        contact_geometry: ContactGeometry::new(1., 1.),
        orientation: PointerOrientation::default(),
        pressure: 1.,
        tangential_pressure: 0.,
        scale_factor,
    }
}

/// All Floem pointer events have a location, but not all baseview ones do,
/// so cache the last position reported, and use that as a value for baseview
/// events that don't provide this information.
struct LastKnownMousePosition {
    // Use atomics as cheap interior mutability
    x: AtomicU64,
    y: AtomicU64,
}

impl LastKnownMousePosition {
    const fn new() -> Self {
        Self {
            x: AtomicU64::new(0),
            y: AtomicU64::new(0),
        }
    }

    fn update(&self, point: &baseview::Point) {
        // Relaxed is fine - we will only ever be called on the UI thread, and that will
        // produce consistent results without any barrier.
        self.x
            .store(point.x.to_bits(), std::sync::atomic::Ordering::Relaxed);
        self.y
            .store(point.y.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }

    fn point(&self) -> baseview::Point {
        baseview::Point {
            x: f64::from_bits(self.x.load(std::sync::atomic::Ordering::Relaxed)),
            y: f64::from_bits(self.y.load(std::sync::atomic::Ordering::Relaxed)),
        }
    }
}
