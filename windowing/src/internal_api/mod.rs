mod window_update;
mod windowing_backend;

pub use window_update::*;
pub use windowing_backend::*;

#[cfg(all(feature = "baseview", not(feature = "winit")))]
pub type WindowingSystem = crate::baseview::Baseview;

#[cfg(all(feature = "winit", not(feature = "baseview")))]
pub type WindowingSystem = crate::winit::WInit;
