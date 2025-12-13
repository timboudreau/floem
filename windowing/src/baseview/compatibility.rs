use baseview_raw_window_handle::{
    RawDisplayHandle as BaseviewRawDisplayHandle, RawWindowHandle as BaseviewRawWindowHandle
};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::ptr::NonNull;

pub(crate) trait Convert<T> {
    fn convert(&self) -> T;
}

#[allow(unused)]
impl Convert<BaseviewRawWindowHandle> for RawWindowHandle {
    fn convert(&self) -> BaseviewRawWindowHandle {
        match self {
            RawWindowHandle::UiKit(ui_kit_window_handle) => {
                let mut result = baseview_raw_window_handle::UiKitWindowHandle::empty();
                result.ui_view = ui_kit_window_handle.ui_view.as_ptr();
                if let Some(controller) = ui_kit_window_handle.ui_view_controller {
                    result.ui_view_controller = controller.as_ptr();
                }
                BaseviewRawWindowHandle::UiKit(result)
            }
            RawWindowHandle::AppKit(app_kit_window_handle) => {
                let mut result = baseview_raw_window_handle::AppKitWindowHandle::empty();
                result.ns_view = app_kit_window_handle.ns_view.as_ptr();
                BaseviewRawWindowHandle::AppKit(result)
            }
            // pending: implement at least these
            RawWindowHandle::Xlib(xlib_window_handle) => todo!(),
            RawWindowHandle::Xcb(xcb_window_handle) => todo!(),
            RawWindowHandle::Wayland(wayland_window_handle) => todo!(),
            RawWindowHandle::Drm(drm_window_handle) => todo!(),
            RawWindowHandle::Win32(win32_window_handle) => todo!(),
            _ => todo!(),
        }
    }
}

#[allow(unused)]
impl Convert<RawWindowHandle> for BaseviewRawWindowHandle {
    fn convert(&self) -> RawWindowHandle {
        match self {
            BaseviewRawWindowHandle::UiKit(ui_kit_window_handle) => {
                let mut result = raw_window_handle::UiKitWindowHandle::new(
                    NonNull::new(ui_kit_window_handle.ui_view).unwrap(),
                );
                if !ui_kit_window_handle.ui_view_controller.is_null() {
                    result.ui_view_controller = unsafe {
                        Some(NonNull::new_unchecked(
                            ui_kit_window_handle.ui_view_controller,
                        ))
                    }
                }
                // pending, other fields
                RawWindowHandle::UiKit(result)
            }
            BaseviewRawWindowHandle::AppKit(app_kit_window_handle) => {
                let mut result = raw_window_handle::AppKitWindowHandle::new(
                    NonNull::new(app_kit_window_handle.ns_window).unwrap(),
                );
                if !app_kit_window_handle.ns_view.is_null() {
                    result.ns_view =
                        unsafe { NonNull::new_unchecked(app_kit_window_handle.ns_view) }
                }
                RawWindowHandle::AppKit(result)
            }
            // pending: implement at least these
            BaseviewRawWindowHandle::Win32(win32_window_handle) => todo!(),
            BaseviewRawWindowHandle::Xlib(xlib_window_handle) => todo!(),
            BaseviewRawWindowHandle::Wayland(wayland_window_handle) => todo!(),
            BaseviewRawWindowHandle::Drm(drm_window_handle) => todo!(),
            _ => todo!(),
        }
    }
}

#[allow(unused)]
impl Convert<BaseviewRawDisplayHandle> for RawDisplayHandle {
    fn convert(&self) -> BaseviewRawDisplayHandle {
        match self {
            RawDisplayHandle::UiKit(ui_kit_display_handle) => {
                let res = baseview_raw_window_handle::UiKitDisplayHandle::empty();
                BaseviewRawDisplayHandle::UiKit(res)
            }
            RawDisplayHandle::AppKit(app_kit_display_handle) => {
                // No fields?  Is this even a thing that does anything?
                let res = baseview_raw_window_handle::AppKitDisplayHandle::empty();
                BaseviewRawDisplayHandle::AppKit(res)
            }
            _ => todo!(),
        }
    }
}

#[allow(unused)]
impl Convert<RawDisplayHandle> for BaseviewRawDisplayHandle {
    fn convert(&self) -> RawDisplayHandle {
        match self {
            BaseviewRawDisplayHandle::UiKit(ui_kit_display_handle) => {
                let mut result = raw_window_handle::UiKitDisplayHandle::new();
                RawDisplayHandle::UiKit(result)
            },
            BaseviewRawDisplayHandle::AppKit(app_kit_display_handle) => {
                let result = raw_window_handle::AppKitDisplayHandle::new();
                RawDisplayHandle::AppKit(result)
            },
            _ => todo!(),
        }
    }
}
