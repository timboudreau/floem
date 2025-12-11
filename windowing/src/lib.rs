mod common;

#[cfg(all(feature = "baseview", not(feature = "winit")))]
mod baseview;

#[cfg(all(feature = "winit", not(feature = "baseview")))]
mod winit;

#[cfg(all(feature = "baseview", not(feature = "winit")))]
pub use crate::baseview::*;

#[cfg(all(feature = "winit", not(feature = "baseview")))]
pub use crate::winit::*;

pub use common::*;

#[cfg(all(feature = "winit", feature = "baseview"))]
compile_error!("feature \"winit\" and feature \"baseview\" are mutually exclusive");
