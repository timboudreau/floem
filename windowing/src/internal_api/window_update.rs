use peniko::kurbo::{Point, Rect, Size};
#[cfg(feature = "winit")]
use winit::window::UserAttentionType;

/// Enum of state updates that can be requested on a window which are processed
/// asynchronously after event processing.
#[allow(dead_code)] // DocumentEdited is seen as unused on non-mac builds
pub enum WindowUpdate {
    Visibility(bool),
    InnerBounds(Rect),
    OuterBounds(Rect),
    // Since both inner bounds and outer bounds require some fudgery because winit
    // only supports setting outer location and *inner* bounds, it is a good idea
    // also to support setting the two things winit supports directly:
    OuterLocation(Point),
    InnerSize(Size),
    #[cfg(feature = "winit")]
    RequestAttention(Option<UserAttentionType>),
    Minimize(bool),
    Maximize(bool),
    // macOS only
    #[allow(unused_variables)] // seen as unused on linux, etc.
    DocumentEdited(bool),
}
