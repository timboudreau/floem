use std::{cell::RefCell, collections::HashMap};
use crate::{public_api::WindowIdentifier, internal_api::WindowUpdate};

// Using thread_local for consistency with static vars in updates.rs, but I suspect these
// are thread_local not because thread-locality is desired, but only because static mutability is
// desired - but that's a patch for another day.
thread_local! {
    /// Holding pen for window state changes, processed as part of the event loop cycle
    pub(crate) static WINDOW_UPDATE_MESSAGES: RefCell<HashMap<WindowIdentifier, Vec<WindowUpdate>>> = Default::default();
}

pub(crate) fn retreive_window_update_messages(id: &WindowIdentifier) -> Option<Vec<WindowUpdate>> {
    WINDOW_UPDATE_MESSAGES.with_borrow_mut(|map| map.remove(id))
}

pub(crate) fn push_window_update_message(id: &WindowIdentifier, msg: WindowUpdate) {
    WINDOW_UPDATE_MESSAGES.with_borrow_mut(|map| match map.entry(*id) {
        std::collections::hash_map::Entry::Occupied(updates) => {
            updates.into_mut().push(msg);
        }
        std::collections::hash_map::Entry::Vacant(v) => {
            v.insert(vec![msg]);
        }
    });
}
