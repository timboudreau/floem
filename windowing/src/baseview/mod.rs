mod backend;
mod compatibility;
mod window_identifier;
mod handles;

use std::sync::Arc;
pub use window_identifier::*;
pub use backend::*;
pub use handles::*;
pub use window_identifier::*;

pub type NativeWindowInner = BaseviewHandles;
pub type NativeWindow = Arc<NativeWindowInner>;
