#[cfg(all(feature = "winit", feature = "baseview"))]
compile_error!("feature \"winit\" and feature \"baseview\" are mutually exclusive");

#[cfg(any(feature = "winit", feature = "baseview"))]
mod common;

#[cfg(any(feature = "winit", feature = "baseview"))]
pub(crate) mod private;

#[cfg(all(feature = "baseview", not(feature = "winit")))]
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
