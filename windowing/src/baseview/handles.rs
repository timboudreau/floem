#![allow(deprecated)]
use std::ops::Deref;
use baseview::Window;
use baseview_raw_window_handle::{
    HasRawWindowHandle as BaseviewHasRawWindowHandle,
    RawWindowHandle as BaseviewRawWindowHandle,
    HasRawDisplayHandle as BaseviewHasRawDisplayHandle,
    RawDisplayHandle as BaseviewRawDisplayHandle,
    HasDisplayHandle as BaseviewHasDisplayHandle,
    HasWindowHandle as BaseviewHasWindowHandle
};
use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle};

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

impl HasWindowHandle for BaseviewHandles {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        Ok(unsafe { WindowHandle::borrow_raw(self.window) })
    }
}

impl HasDisplayHandle for BaseviewHandles {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(unsafe { DisplayHandle::borrow_raw(self.display)})
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

impl BaseviewHasDisplayHandle for BaseviewHandles {
    fn display_handle(&self) -> Result<baseview_raw_window_handle::DisplayHandle<'_>, baseview_raw_window_handle::HandleError> {
        Ok(unsafe { baseview_raw_window_handle::DisplayHandle::borrow_raw(self.display.convert()) })
    }
}

impl BaseviewHasWindowHandle for BaseviewHandles {
    fn window_handle(&self) -> Result<baseview_raw_window_handle::WindowHandle<'_>, baseview_raw_window_handle::HandleError> {
        let h : BaseviewRawWindowHandle = self.window.convert();
        Ok(unsafe { baseview_raw_window_handle::WindowHandle::borrow_raw(h, baseview_raw_window_handle::ActiveHandle::new()) })
    }
}

unsafe impl Send for BaseviewHandles{}
unsafe impl Sync for BaseviewHandles{}
