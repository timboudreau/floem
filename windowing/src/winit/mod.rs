mod backend;
mod window_geometry;
mod window_identifier;
mod winit_screen_layout;

use std::sync::Arc;
use winit::window::Window;

pub use backend::*;
pub use window_identifier::*;

pub type NativeWindow = Arc<dyn Window>;
