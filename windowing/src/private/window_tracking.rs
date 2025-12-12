//! Provides tracking to map window ids to windows in order to force repaints
//! on inactive windows (which would otherwise not receive messages), and so
//! that views can retrieve the `WindowId` of the window that contains them
//! and use the methods that look up the `Window` for that id to retrieve information
//! such as screen position.
use crate::{
    internal_api::{WindowingBackend, WindowingSystem},
    public_api::{NativeWindow, ViewId, WindowIdentifier},
};
use std::collections::HashMap;

#[cfg(all(feature = "winit", not(feature = "baseview")))]
mod mapping_storage {
    use crate::private::window_tracking::WindowMapping;
    use std::sync::{OnceLock, RwLock};
    static WINDOW_FOR_WINDOW_AND_ROOT_IDS: OnceLock<RwLock<WindowMapping>> = OnceLock::new();

    pub(crate) fn with_window_map_mut<F: FnMut(&mut WindowMapping)>(mut f: F) -> bool {
        let map = WINDOW_FOR_WINDOW_AND_ROOT_IDS.get_or_init(|| RwLock::new(Default::default()));
        if let Ok(mut map) = map.write() {
            f(&mut map);
            true
        } else {
            false
        }
    }

    pub(crate) fn with_window_map<F: FnOnce(&WindowMapping) -> T, T>(f: F) -> Option<T> {
        let map = WINDOW_FOR_WINDOW_AND_ROOT_IDS.get_or_init(|| RwLock::new(Default::default()));
        if let Ok(map) = map.read() {
            Some(f(&map))
        } else {
            None
        }
    }
}

#[cfg(all(feature = "baseview", not(feature = "winit")))]
mod mapping_storage {
    use crate::private::window_tracking::WindowMapping;
    use std::cell::RefCell;
    thread_local! {
        static WINDOW_FOR_WINDOW_AND_ROOT_IDS: RefCell<WindowMapping> = RefCell::default();
    }

    pub(crate) fn with_window_map_mut<F: FnMut(&mut WindowMapping)>(f: F) -> bool {
        WINDOW_FOR_WINDOW_AND_ROOT_IDS.with_borrow_mut(f);
        // unlike the winit version, this cannot fail, but we should make the implementations compatible.
        true
    }

    pub(crate) fn with_window_map<F: FnOnce(&WindowMapping) -> T, T>(f: F) -> Option<T> {
        Some(WINDOW_FOR_WINDOW_AND_ROOT_IDS.with_borrow(f))
    }
}

/// Add a mapping from `root_id` -> `window_id` -> `window` for the given triple.
pub fn store_window_id_mapping(
    root_id: ViewId,
    window_id: WindowIdentifier,
    window: &NativeWindow,
) {
    with_window_map_mut(move |m| m.add(root_id, window_id, window.clone()));
}

/// Remove the mapping from `root_id` -> `window_id` -> `window` for the given triple.
pub fn remove_window_id_mapping(root_id: &ViewId, window_id: &WindowIdentifier) {
    with_window_map_mut(move |m| m.remove(root_id, window_id));
}

/// Maps root-id:window-id:window triples, so a view can get its root and
/// from that locate the window-id (if any) that it belongs to.
#[derive(Default, Clone)]
pub(crate) struct WindowMapping {
    pub(crate) window_for_window_id: HashMap<WindowIdentifier, NativeWindow>,
    pub(crate) window_id_for_root_view_id: HashMap<ViewId, WindowIdentifier>,
}

impl WindowMapping {
    pub fn add(
        &mut self,
        root: ViewId,
        window_id: impl Into<WindowIdentifier>,
        window: NativeWindow,
    ) {
        let id = window_id.into();
        self.window_for_window_id.insert(id, window);
        self.window_id_for_root_view_id.insert(root, id);
    }

    pub fn remove(&mut self, root: &ViewId, window_id: &WindowIdentifier) {
        let root_found = self.window_id_for_root_view_id.remove(root).is_some();
        let window_found = self.window_for_window_id.remove(window_id).is_some();
        debug_assert!(
            root_found == window_found,
            "Window mapping state inconsistent. Remove root {root:?} success was {root_found} but remove {window_id:?} success was {window_found}"
        );
    }

    pub fn with_window_id_and_window<F: FnOnce(&WindowIdentifier, &NativeWindow) -> T, T>(
        &self,
        root_view_id: ViewId,
        f: F,
    ) -> Option<T> {
        self.window_id_for_root_view_id
            .get(&root_view_id)
            .and_then(|window_id| {
                self.window_for_window_id
                    .get(window_id)
                    .map(|window| f(window_id, window))
            })
    }

    pub fn with_window<F: FnOnce(&NativeWindow) -> T, T>(
        &self,
        window: &WindowIdentifier,
        f: F,
    ) -> Option<T> {
        self.window_for_window_id.get(window).map(f)
    }

    pub fn window_id_for_root(&self, id: &ViewId) -> Option<WindowIdentifier> {
        self.window_id_for_root_view_id.get(id).copied()
    }

    pub fn root_view_id_for(&self, window_id: &WindowIdentifier) -> Option<ViewId> {
        for (k, v) in self.window_id_for_root_view_id.iter() {
            if v == window_id {
                return Some(*k);
            }
        }
        None
    }
}

pub fn with_window_id_and_window<F: FnOnce(&WindowIdentifier, &NativeWindow) -> T, T>(
    view: &ViewId,
    f: F,
) -> Option<T> {
    // view.root()
    WindowingSystem::root_of(view)
        .and_then(|root_view_id| with_window_map(|m| m.with_window_id_and_window(root_view_id, f)))
        .unwrap_or(None)
}

pub fn is_known_root(id: &ViewId) -> bool {
    with_window_map(|map| map.window_id_for_root_view_id.contains_key(id)).unwrap_or(false)
}

fn with_window_map_mut<F: FnMut(&mut WindowMapping)>(f: F) -> bool {
    mapping_storage::with_window_map_mut(f)
}

pub fn with_window_map<F: FnOnce(&WindowMapping) -> T, T>(f: F) -> Option<T> {
    mapping_storage::with_window_map(f)
}

pub fn with_window<F: FnOnce(&NativeWindow) -> T, T>(window: &WindowIdentifier, f: F) -> Option<T> {
    with_window_map(|m| m.with_window(window, |w| f(w))).unwrap_or(None)
}

pub fn root_view_id(window: &WindowIdentifier) -> Option<ViewId> {
    with_window_map(|m| m.root_view_id_for(window)).unwrap_or(None)
}

/// Force a single window to repaint - this is necessary in cases where the
/// window is not the active window and otherwise would not process update
/// messages sent to it.
#[cfg(all(feature = "winit", not(feature = "baseview")))]
pub fn force_window_repaint(id: &WindowIdentifier) -> bool {
    with_window_map(|m| {
        m.with_window(id, |window| window.request_redraw())
            .is_some()
    })
    .unwrap_or(false)
}

pub fn window_id_for_root(root_id: ViewId) -> Option<WindowIdentifier> {
    with_window_map(|map| map.window_id_for_root(&root_id)).unwrap_or(None)
}
