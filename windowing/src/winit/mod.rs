mod window_identifier;

use std::sync::Arc;
use winit::window::Window;

pub use window_identifier::*;

pub type NativeWindow = Arc<dyn Window>;
