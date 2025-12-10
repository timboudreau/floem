use std::{
    ffi::c_void,
    num::{NonZeroIsize, NonZeroU32},
    ptr::NonNull,
};

use baseview::Window;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use raw_window_handle_06::{
    HasRawWindowHandle as HasRawWindowHandle06, RawWindowHandle as RawWindowHandle06,
};

/*
Cribbed from nih-plug in plugins/examples/byo_gui_wgpu/src/lib.rs
*/
pub(crate) fn baseview_window_to_surface_target(
    window: &baseview::Window<'_>,
) -> wgpu::SurfaceTargetUnsafe {
    use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

    let raw_display_handle = window.raw_display_handle();
    let raw_window_handle = window.raw_window_handle();

    wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: match raw_display_handle {
            raw_window_handle::RawDisplayHandle::AppKit(_) => {
                raw_window_handle_06::RawDisplayHandle::AppKit(
                    raw_window_handle_06::AppKitDisplayHandle::new(),
                )
            }
            raw_window_handle::RawDisplayHandle::Xlib(handle) => {
                raw_window_handle_06::RawDisplayHandle::Xlib(
                    raw_window_handle_06::XlibDisplayHandle::new(
                        NonNull::new(handle.display),
                        handle.screen,
                    ),
                )
            }
            raw_window_handle::RawDisplayHandle::Xcb(handle) => {
                raw_window_handle_06::RawDisplayHandle::Xcb(
                    raw_window_handle_06::XcbDisplayHandle::new(
                        NonNull::new(handle.connection),
                        handle.screen,
                    ),
                )
            }
            raw_window_handle::RawDisplayHandle::Windows(_) => {
                raw_window_handle_06::RawDisplayHandle::Windows(
                    raw_window_handle_06::WindowsDisplayHandle::new(),
                )
            }
            _ => todo!(),
        },
        raw_window_handle: match raw_window_handle {
            raw_window_handle::RawWindowHandle::AppKit(handle) => {
                raw_window_handle_06::RawWindowHandle::AppKit(
                    raw_window_handle_06::AppKitWindowHandle::new(
                        NonNull::new(handle.ns_view).unwrap(),
                    ),
                )
            }
            raw_window_handle::RawWindowHandle::Xlib(handle) => {
                raw_window_handle_06::RawWindowHandle::Xlib(
                    raw_window_handle_06::XlibWindowHandle::new(handle.window),
                )
            }
            raw_window_handle::RawWindowHandle::Xcb(handle) => {
                raw_window_handle_06::RawWindowHandle::Xcb(
                    raw_window_handle_06::XcbWindowHandle::new(
                        NonZeroU32::new(handle.window).unwrap(),
                    ),
                )
            }
            raw_window_handle::RawWindowHandle::Win32(handle) => {
                let mut raw_handle = raw_window_handle_06::Win32WindowHandle::new(
                    NonZeroIsize::new(handle.hwnd as isize).unwrap(),
                );

                raw_handle.hinstance = NonZeroIsize::new(handle.hinstance as isize);

                raw_window_handle_06::RawWindowHandle::Win32(raw_handle)
            }
            _ => todo!(),
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParentWindowHandle {
    /// The ID of the host's parent window. Used with X11.
    X11Window(u32),
    /// A handle to the host's parent window. Used only on macOS.
    AppKitNsView(*mut c_void),
    /// A handle to the host's parent window. Used only on Windows.
    Win32Hwnd(*mut c_void),
}

unsafe impl HasRawWindowHandle for ParentWindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        match *self {
            ParentWindowHandle::X11Window(window) => {
                let mut handle = raw_window_handle::XcbWindowHandle::empty();
                handle.window = window;
                RawWindowHandle::Xcb(handle)
            }
            ParentWindowHandle::AppKitNsView(ns_view) => {
                let mut handle = raw_window_handle::AppKitWindowHandle::empty();
                handle.ns_view = ns_view;
                RawWindowHandle::AppKit(handle)
            }
            ParentWindowHandle::Win32Hwnd(hwnd) => {
                let mut handle = raw_window_handle::Win32WindowHandle::empty();
                handle.hwnd = hwnd;
                RawWindowHandle::Win32(handle)
            }
        }
    }
}
/*
impl HasRawWindowHandle06 for ParentWindowHandle {
    fn raw_window_handle(&self) -> Result<RawWindowHandle06, raw_window_handle_06::HandleError> {
        match *self {
            ParentWindowHandle::X11Window(window) => {
                let mut handle = raw_window_handle_06::XcbWindowHandle::empty();
                handle.window = window;
                RawWindowHandle06::Xcb(handle)
            }
            ParentWindowHandle::AppKitNsView(ns_view) => {
                let mut handle = raw_window_handle_06::AppKitWindowHandle::;
                handle.ns_view = ns_view;
                RawWindowHandle06::AppKit(handle)
            }
            ParentWindowHandle::Win32Hwnd(hwnd) => {
                let mut handle = raw_window_handle_06::Win32WindowHandle::empty();
                handle.hwnd = hwnd;
                RawWindowHandle06::Win32(handle)
            }
        }
    }
}
 */

impl<'l> From<&Window<'l>> for ParentWindowHandle {
    fn from(window: &Window) -> Self {
        match window.raw_window_handle() {
            raw_window_handle::RawWindowHandle::Xlib(handle) => {
                ParentWindowHandle::X11Window(handle.window as u32)
            }
            raw_window_handle::RawWindowHandle::Xcb(handle) => {
                ParentWindowHandle::X11Window(handle.window)
            }
            raw_window_handle::RawWindowHandle::AppKit(handle) => {
                ParentWindowHandle::AppKitNsView(handle.ns_view)
            }
            raw_window_handle::RawWindowHandle::Win32(handle) => {
                ParentWindowHandle::Win32Hwnd(handle.hwnd)
            }
            handle => unimplemented!("Unsupported window handle: {handle:?}"),
        }
    }
}
