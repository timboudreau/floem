mod backend;
mod compatibility;
mod window_identifier;

use raw_window_handle::*;
use std::sync::Arc;

/*
Pending - we need a way to hide the lifetime on baseview's Window type, or find a way
to remove it in Baseview (I don't see any trivial way of doing that, but it's not clear what
owns the `WindowInner` - probably the host application, in which case it is effectively static
- it *must* outlive whatever is drawing in it).
*/

pub use window_identifier::*;
pub use backend::*;

pub type NativeWindowInner = RawWindowHandle;
pub type NativeWindow = Arc<NativeWindowInner>;
