use peniko::Color;


// Windows specific window configuration properties.
#[derive(Debug, Clone)]
pub struct WinOSWindowConfig {
    // pub(crate) top_resize_border: bool,
    pub(crate) corner_preference: WinOsCornerPreference,
    pub(crate) set_enable: bool,
    pub(crate) set_skip_taskbar: bool,
    // /// Shows or hides the background drop shadow for undecorated windows.
    // ///
    // /// Enabling the shadow causes a thin 1px line to appear on the top of the window.
    // pub(crate) set_undecorated_shadow: bool,
    pub(crate) set_system_backdrop: WinOsBackdropType,
    pub(crate) set_border_color: Option<Color>,
    pub(crate) set_title_background_color: Option<Color>,
    pub(crate) set_title_text_color: Option<Color>,
}

impl Default for WinOSWindowConfig {
    fn default() -> Self {
        Self {
            // top_resize_border: true,
            corner_preference: WinOsCornerPreference::Default,
            set_enable: true,
            set_skip_taskbar: false,
            set_system_backdrop: WinOsBackdropType::Auto,
            set_border_color: None,
            set_title_background_color: None,
            set_title_text_color: None,
        }
    }
}

impl WinOSWindowConfig {
    // TODO: need to patch winit first
    // /// Turn window top resize border on or off (for windows without a title bar).
    // /// By default this is enabled.
    // pub fn top_resize_border(mut self, top_resize_border: bool) -> Self {
    //     self.top_resize_border = top_resize_border;
    //     self
    // }

    /// Sets the preferred style of the window corners.
    ///
    /// Supported starting with Windows 11 Build 22000.
    pub fn corner_preference(mut self, corner_preference: WinOsCornerPreference) -> Self {
        self.corner_preference = corner_preference;
        self
    }

    /// Enables or disables mouse and keyboard input to the specified window.
    ///
    /// A window must be enabled before it can be activated.
    /// If an application has create a modal dialog box by disabling its owner window
    /// (as described in [`WindowAttributesExtWindows::with_owner_window`]), the application must
    /// enable the owner window before destroying the dialog box.
    /// Otherwise, another window will receive the keyboard focus and be activated.
    ///
    /// If a child window is disabled, it is ignored when the system tries to determine which
    /// window should receive mouse messages.
    ///
    /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enablewindow#remarks>
    /// and <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#disabled-windows>.
    pub fn set_enable(mut self, set_enable: bool) -> Self {
        self.set_enable = set_enable;
        self
    }

    /// Whether to show or hide the window icon in the taskbar.
    pub fn set_skip_taskbar(mut self, set_skip_taskbar: bool) -> Self {
        self.set_skip_taskbar = set_skip_taskbar;
        self
    }

    /// Sets system-drawn backdrop type.
    ///
    /// Requires Windows 11 build 22523+.
    pub fn set_system_backdrop(mut self, set_system_backdrop: WinOsBackdropType) -> Self {
        self.set_system_backdrop = set_system_backdrop;
        self
    }

    /// Sets the color of the window border.
    ///
    /// Supported starting with Windows 11 Build 22000.
    pub fn set_border_color(mut self, set_border_color: Color) -> Self {
        self.set_border_color = Some(set_border_color);
        self
    }

    /// Sets the background color of the title bar.
    ///
    /// Supported starting with Windows 11 Build 22000.
    pub fn set_title_background_color(mut self, set_title_background_color: Color) -> Self {
        self.set_title_background_color = Some(set_title_background_color);
        self
    }

    /// Sets the color of the window title.
    ///
    /// Supported starting with Windows 11 Build 22000.
    pub fn set_title_text_color(mut self, set_title_text_color: Color) -> Self {
        self.set_title_text_color = Some(set_title_text_color);
        self
    }
}

/// Describes how the corners of a window should look like.
///
/// For a detailed explanation, see [`DWM_WINDOW_CORNER_PREFERENCE docs`].
///
/// [`DWM_WINDOW_CORNER_PREFERENCE docs`]: https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_window_corner_preference
#[derive(Debug, Clone)]
pub enum WinOsCornerPreference {
    /// Corresponds to `DWMWCP_DEFAULT`.
    ///
    /// Let the system decide when to round window corners.
    Default,

    /// Corresponds to `DWMWCP_DONOTROUND`.
    ///
    /// Never round window corners.
    DoNotRound,

    /// Corresponds to `DWMWCP_ROUND`.
    ///
    /// Round the corners, if appropriate.
    Round,

    /// Corresponds to `DWMWCP_ROUNDSMALL`.
    ///
    /// Round the corners if appropriate, with a small radius.
    RoundSmall,
}

#[cfg(target_os = "windows")]
impl From<WinOsCornerPreference> for winit::platform::windows::CornerPreference {
    fn from(value: WinOsCornerPreference) -> Self {
        use winit::platform::windows::CornerPreference;
        match value {
            WinOsCornerPreference::Default => CornerPreference::Default,
            WinOsCornerPreference::DoNotRound => CornerPreference::DoNotRound,
            WinOsCornerPreference::Round => CornerPreference::Round,
            WinOsCornerPreference::RoundSmall => CornerPreference::RoundSmall,
        }
    }
}

/// Describes a system-drawn backdrop material of a window.
///
/// For a detailed explanation, see [`DWM_SYSTEMBACKDROP_TYPE docs`].
///
/// [`DWM_SYSTEMBACKDROP_TYPE docs`]: https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type
#[derive(Debug, Clone)]
pub enum WinOsBackdropType {
    /// Corresponds to `DWMSBT_AUTO`.
    ///
    /// Usually draws a default backdrop effect on the title bar.
    Auto,

    /// Corresponds to `DWMSBT_NONE`.
    None,

    /// Corresponds to `DWMSBT_MAINWINDOW`.
    ///
    /// Draws the Mica backdrop material.
    MainWindow,

    /// Corresponds to `DWMSBT_TRANSIENTWINDOW`.
    ///
    /// Draws the Background Acrylic backdrop material.
    TransientWindow,

    /// Corresponds to `DWMSBT_TABBEDWINDOW`.
    ///
    /// Draws the Alt Mica backdrop material.
    TabbedWindow,
}

#[cfg(target_os = "windows")]
impl From<WinOsBackdropType> for winit::platform::windows::BackdropType {
    fn from(value: WinOsBackdropType) -> Self {
        use winit::platform::windows::BackdropType;
        match value {
            WinOsBackdropType::Auto => BackdropType::Auto,
            WinOsBackdropType::None => BackdropType::None,
            WinOsBackdropType::MainWindow => BackdropType::MainWindow,
            WinOsBackdropType::TransientWindow => BackdropType::TransientWindow,
            WinOsBackdropType::TabbedWindow => BackdropType::TabbedWindow,
        }
    }
}

#[cfg(target_os = "windows")]
pub(super) fn convert_to_win(c: Option<peniko::Color>) -> Option<winit::platform::windows::Color> {
    c.map(|c| {
        let c = c.to_rgba8();
        winit::platform::windows::Color::from_rgb(c.r, c.g, c.b)
    })
}
