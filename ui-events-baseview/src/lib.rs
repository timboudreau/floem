pub mod pointer;
pub mod window;

use baseview::Event;
use ui_events::{
    keyboard::KeyboardEvent,
    pointer::{PointerEvent, PointerState},
};

use crate::pointer::convert_mouse_event;
/*
pub fn convert_event(event: &Event) -> Option<event::Event> {
    match event {
        Event::Mouse(mouse_event) => todo!(),
        Event::Keyboard(keyboard_event) => todo!(),
        Event::Window(window_event) => todo!(),
    }
}
 */

#[derive(Debug, Default)]
pub struct WindowEventReducer {
    /// State of modifiers.
    // modifiers: ModifiersState,
    /// State of the primary mouse pointer.
    primary_state: PointerState,
    // Click and tap counter.
    // counter: TapCounter,
    // First time an event was received..
    // first_instant: Option<Instant>,
}

#[allow(clippy::cast_possible_truncation)]
impl WindowEventReducer {
    /// Process an [`Event`].
    pub fn reduce(&mut self, scale_factor: f64, event: &Event) -> Option<WindowEventTranslation> {
        self.primary_state.scale_factor = scale_factor;

        match event {
            Event::Mouse(mouse_event) => convert_mouse_event(mouse_event, scale_factor)
                .map(|e| WindowEventTranslation::Pointer(e)),
            Event::Keyboard(keyboard_event) => {
                Some(WindowEventTranslation::Keyboard(keyboard_event.clone()))
            }
            Event::Window(_) => None,
        }
    }
}

/// Result of [`WindowEventReducer::reduce`].
#[derive(Debug)]
pub enum WindowEventTranslation {
    /// Resulting [`KeyboardEvent`].
    Keyboard(KeyboardEvent),
    /// Resulting [`PointerEvent`].
    Pointer(PointerEvent),
}
