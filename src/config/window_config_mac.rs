
/// Mac-OS specific window configuration properties, accessible via `WindowConfig::with_mac_os_config( FnMut( MacOsWindowConfig ) )`
///
/// See [the winit docs](https://docs.rs/winit/latest/winit/platform/macos/trait.WindowExtMacOS.html) for further
/// information.
#[derive(Default, Debug, Clone)]
pub struct MacOSWindowConfig {
    pub(crate) movable_by_window_background: Option<bool>,
    pub(crate) titlebar_transparent: Option<bool>,
    pub(crate) titlebar_hidden: Option<bool>,
    pub(crate) title_hidden: Option<bool>,
    pub(crate) titlebar_buttons_hidden: Option<bool>,
    pub(crate) full_size_content_view: Option<bool>,
    pub(crate) unified_titlebar: Option<bool>,
    pub(crate) movable: Option<bool>,
    pub(crate) traffic_lights_offset: Option<(f64, f64)>,
    pub(crate) accepts_first_mouse: Option<bool>,
    pub(crate) tabbing_identifier: Option<String>,
    pub(crate) option_as_alt: Option<MacOsOptionAsAlt>,
    pub(crate) has_shadow: Option<bool>,
    pub(crate) disallow_high_dpi: Option<bool>,
    pub(crate) panel: Option<bool>,
}

impl MacOSWindowConfig {
    /// Allow the window to be
    /// [moved by dragging its background](https://developer.apple.com/documentation/appkit/nswindow/1419072-movablebywindowbackground).
    pub fn movable_by_window_background(mut self, val: bool) -> Self {
        self.movable_by_window_background = Some(val);
        self
    }

    /// Make the titlebar's transparency (does nothing on some versions of macOS).
    pub fn transparent_title_bar(mut self, val: bool) -> Self {
        self.titlebar_transparent = Some(val);
        self
    }

    /// Hides the title bar.
    pub fn hide_titlebar(mut self, val: bool) -> Self {
        self.titlebar_hidden = Some(val);
        self
    }

    /// Hides the title.
    pub fn hide_title(mut self, val: bool) -> Self {
        self.title_hidden = Some(val);
        self
    }

    /// Hides the title bar buttons.
    pub fn hide_titlebar_buttons(mut self, val: bool) -> Self {
        self.titlebar_buttons_hidden = Some(val);
        self
    }

    /// Make the window content [use the full size of the window, including the
    /// [title bar area](https://developer.apple.com/documentation/appkit/nswindow/stylemask/1644646-fullsizecontentview).
    pub fn full_size_content_view(mut self, val: bool) -> Self {
        self.full_size_content_view = Some(val);
        self
    }

    /// unify the titlebar
    pub fn unified_titlebar(mut self, val: bool) -> Self {
        self.unified_titlebar = Some(val);
        self
    }

    /// Allow the window to be moved or not.
    pub fn movable(mut self, val: bool) -> Self {
        self.movable = Some(val);
        self
    }

    /// Specify the position of the close / minimize / full screen buttons
    /// on macOS
    pub fn traffic_lights_offset(mut self, x_y_offset: (f64, f64)) -> Self {
        self.traffic_lights_offset = Some(x_y_offset);
        self
    }

    /// Specify that this window should be sent an event for the initial
    /// click in it when it was previously inactive, rather than treating
    /// that click is only activating the window and not forwarding it to
    /// application code.
    pub fn accept_first_mouse(mut self, val: bool) -> Self {
        self.accepts_first_mouse = Some(val);
        self
    }

    /// Give this window an identifier when tabbing between windows.
    pub fn tabbing_identifier(mut self, val: impl Into<String>) -> Self {
        self.tabbing_identifier = Some(val.into());
        self
    }

    /// Specify how the window will treat `Option` keys on the Mac keyboard -
    /// as a compose key for additional characters, or as a modifier key.
    pub fn interpret_option_as_alt(mut self, val: MacOsOptionAsAlt) -> Self {
        self.option_as_alt = Some(val);
        self
    }

    /// Set whether the window should have a shadow.
    pub fn enable_shadow(mut self, val: bool) -> Self {
        self.has_shadow = Some(val);
        self
    }

    /// Set whether the window's coordinate space and painting should
    /// be scaled for the display or pixel-accurate.
    pub fn disallow_high_dpi(mut self, val: bool) -> Self {
        self.disallow_high_dpi = Some(val);
        self
    }

    /// Set whether the window is a panel
    pub fn panel(mut self, val: bool) -> Self {
        self.panel = Some(val);
        self
    }
}

/// macOS specific configuration for how the Option key is treated
///
/// macOS allows altering the way Option and Alt keys so Alt is treated
/// as a modifier key rather than in character compose key.  This is a proxy
/// for winit's [OptionAsAlt](https://docs.rs/winit/latest/winit/platform/macos/enum.OptionAsAlt.html).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacOsOptionAsAlt {
    OnlyLeft,
    OnlyRight,
    Both,
    #[default]
    None,
}

#[cfg(all(target_os = "macos", feature = "winit", not(feature = "baseview")))]
impl From<MacOsOptionAsAlt> for winit::platform::macos::OptionAsAlt {
    fn from(opts: MacOsOptionAsAlt) -> winit::platform::macos::OptionAsAlt {
        match opts {
            MacOsOptionAsAlt::OnlyLeft => winit::platform::macos::OptionAsAlt::OnlyLeft,
            MacOsOptionAsAlt::OnlyRight => winit::platform::macos::OptionAsAlt::OnlyRight,
            MacOsOptionAsAlt::Both => winit::platform::macos::OptionAsAlt::Both,
            MacOsOptionAsAlt::None => winit::platform::macos::OptionAsAlt::None,
        }
    }
}
