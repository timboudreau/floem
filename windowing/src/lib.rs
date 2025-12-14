#[cfg(all(feature = "winit", feature = "baseview"))]
compile_error!("feature \"winit\" and feature \"baseview\" are mutually exclusive");
#[cfg(all(not(feature = "baseview"), not(feature = "winit")))]
compile_error!("One of the features `baseview` or `winit` must be enabled.");

#[cfg(any(feature = "winit", feature = "baseview"))]
mod common;

#[cfg(any(feature = "winit", feature = "baseview"))]
pub(crate) mod private;

// #[cfg(all(feature = "baseview", not(feature = "winit")))]
#[cfg(feature = "baseview")]
mod baseview;

#[cfg(all(feature = "winit", not(feature = "baseview")))]
mod winit;

#[cfg(any(feature = "winit", feature = "baseview"))]
pub mod internal_api;

pub mod public_api {
    #[cfg(any(feature = "winit", feature = "baseview"))]
    pub use super::common::*;

    #[cfg(all(feature = "baseview", not(feature = "winit")))]
    pub use super::baseview::*;

    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    pub use super::winit::*;
}

// Sigh.
#[cfg(all(feature = "baseview", not(feature = "winit")))]
pub use baseview_raw_window_handle::{HasRawWindowHandle as BaseviewHasRawWindowHandle, RawWindowHandle as BaseviewRawWindowHandle, HasRawDisplayHandle as BaseviewHasRawDisplayHandle, RawDisplayHandle as BaseviewRawDisplayHandle};
