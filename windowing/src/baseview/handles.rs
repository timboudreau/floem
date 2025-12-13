#![allow(deprecated)]

use std::ops::Deref;

use baseview::Window;
use baseview_raw_window_handle::{HasRawWindowHandle as BaseviewHasRawWindowHandle, RawWindowHandle as BaseviewRawWindowHandle, HasRawDisplayHandle as BaseviewHasRawDisplayHandle, RawDisplayHandle as BaseviewRawDisplayHandle};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};

use crate::baseview::compatibility::Convert;

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub struct BaseviewHandles {
    pub(super) window : RawWindowHandle,
    pub(super) display : RawDisplayHandle,
}

impl Eq for BaseviewHandles {}

impl Deref for BaseviewHandles {
    type Target = RawWindowHandle;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl BaseviewHandles {
    pub fn matches<'l>(&self, window : &mut Window<'l>) -> bool {
        let handle: RawWindowHandle = window.raw_window_handle().convert();
        &self.window == &handle
    }
}

unsafe impl HasRawDisplayHandle for BaseviewHandles {
    fn raw_display_handle(&self) -> Result<RawDisplayHandle, raw_window_handle::HandleError> {
        Ok(self.display)
    }
}

unsafe impl HasRawWindowHandle for BaseviewHandles {
    fn raw_window_handle(&self) -> Result<RawWindowHandle, raw_window_handle::HandleError> {
        Ok(self.window)
    }
}

unsafe impl BaseviewHasRawWindowHandle for BaseviewHandles {
    fn raw_window_handle(&self) -> BaseviewRawWindowHandle {
        self.window.convert()
    }
}

unsafe impl BaseviewHasRawDisplayHandle for BaseviewHandles {
    fn raw_display_handle(&self) -> BaseviewRawDisplayHandle {
        self.display.convert()
    }
}

unsafe impl Send for BaseviewHandles{}
unsafe impl Sync for BaseviewHandles{}
