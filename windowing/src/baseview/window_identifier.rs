#![allow(deprecated)]
use baseview::*;
use baseview_raw_window_handle::{
    HasRawWindowHandle as BaseviewHasRawWindowHandle, RawWindowHandle as BaseviewRawWindowHandle,
    HasRawDisplayHandle as BaseviewHasRawDisplayHandle
};
use slotmap::*;
use std::cell::RefCell;

use crate::baseview::{compatibility::Convert};
use super::handles::BaseviewHandles;
/*
Hmm, baseview::Window has a lifetime.

Maybe use slotmap to manage the instance?  Or cast to 'static and cheat - in theory,
at least the main window must outlive any rust code rendering into it.  A dynamically
created popup window is more questionable.  And more than one instance will not have the
same lifetime, so there may be no avoiding some transmuting.

A real solution would be to eliminate the lifetime in baseview; it appears it is only
needed for Windows, and the thing that actually uses the lifetime a few layers down is
the os-allocated window handle.  Lifetimes have limited meaning for things like that,
and it might as well be a pointer because in reality, Rust code knows nothing about when
the host application or OS might choose to deallocate it - its allocation is beyond Rust's
control in the first place.
*/

new_key_type! {
    /// A small unique identifier for an instance of a `View`.
    ///
    /// This id is how you can access and modify a view, including accessing children views and updating state.
   pub struct WindowIdentifier;
}

impl crate::common::WindowIdDelegate for WindowIdentifier {}

thread_local! {
    static WINDOW_STORAGE : RefCell<SlotMap<WindowIdentifier, BaseviewHandles>> = RefCell::new(SlotMap::with_key());
}

/// In theory, if the window was initially passed without native resources allocated,
/// then we could update them here. Or conceivably, the window could be replaced at runtime?
pub fn update_registration<'a>(id: WindowIdentifier, window : &mut Window<'a>) {
    let handles = BaseviewHandles {
        window : window.raw_window_handle().convert(),
        display : window.raw_display_handle().convert(),
    };
    WINDOW_STORAGE.with(|cell| {
        let mut m = cell.borrow_mut();
        if let Some(r) = m.get_mut(id) {
            *r = handles;
        } else {
            panic!("Updating registration for an id not present.");
        }
    });
}

pub fn register_window<'a>(window: &mut Window<'a>) -> WindowIdentifier {
    let handles = BaseviewHandles {
        window : window.raw_window_handle().convert(),
        display : window.raw_display_handle().convert(),
    };
    WINDOW_STORAGE.with(|cell| cell.borrow_mut().insert(handles))
}

/// Not sure if we will need reverse lookup, but it is helpful for debugging
pub fn find(window : &mut Window<'_>) -> Option<WindowIdentifier> {
    let handle = window.raw_window_handle();
    window_id_for(&handle)
}

pub fn window_id_for(handle: &BaseviewRawWindowHandle) -> Option<WindowIdentifier> {
    let ours = handle.convert();
    WINDOW_STORAGE.with(|cell| {
        for (k, v) in cell.borrow().iter() {
            if &v.window == &ours {
                return Some(k);
            }
        }
        return None;
    })
}
