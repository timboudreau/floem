#[cfg(feature = "winit")]
use winit::window::UserAttentionType;

/// Delegate enum for `winit`'s [`UserAttentionType`](https://docs.rs/winit/latest/winit/window/enum.UserAttentionType.html)
///
/// This is used for making the window's icon bounce in the macOS dock or the equivalent of that on
/// other platforms.
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Urgency {
    Critical,
    Informational,

    /// The default attention type (equivalent of passing `None` to `winit::Window::request_user_attention())`).
    /// On some platforms (X11), it is necessary to call `WindowIdentifier.request_attention(Urgency::Default)` to stop
    /// the attention-seeking behavior of the window.
    #[default]
    Default,
}

impl Urgency {
    pub fn is_stop(&self) -> bool {
        matches![self, Self::Default]
    }
}

#[cfg(feature = "winit")]
impl From<Urgency> for Option<UserAttentionType> {
    fn from(urgency: Urgency) -> Self {
        match urgency {
            Urgency::Critical => Some(UserAttentionType::Critical),
            Urgency::Informational => Some(UserAttentionType::Informational),
            Urgency::Default => None,
        }
    }
}

#[cfg(feature = "winit")]
impl From<UserAttentionType> for Urgency {
    fn from(attention_type: UserAttentionType) -> Self {
        match attention_type {
            UserAttentionType::Critical => Self::Critical,
            UserAttentionType::Informational => Self::Informational,
        }
    }
}
