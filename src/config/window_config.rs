use adapters::WindowSystemTheme;
use peniko::kurbo::{Point, Size};
#[cfg(all(feature = "winit", not(feature = "baseview")))]
use winit::{icon::Icon, monitor::Fullscreen, window::{WindowButtons, WindowLevel}};
use crate::window::{MacOSWindowConfig, WebWindowConfig, WinOSWindowConfig};

/// Configures various attributes (e.g. size, position, transparency, etc.) of a window.
pub struct WindowConfig {
    pub(crate) size: Option<Size>,
    pub(crate) min_size: Option<Size>,
    pub(crate) max_size: Option<Size>,
    pub(crate) position: Option<Point>,
    pub(crate) show_titlebar: bool,
    pub(crate) transparent: bool,
    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    pub(crate) fullscreen: Option<Fullscreen>,
    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    pub(crate) window_icon: Option<Icon>,
    pub(crate) title: String,
    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    pub(crate) enabled_buttons: WindowButtons,
    pub(crate) resizable: bool,
    pub(crate) undecorated: bool,
    pub(crate) undecorated_shadow: bool,
    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    pub(crate) window_level: WindowLevel,
    /// Applies chosen theme or os theme, when `None` is provided.
    pub(crate) theme_override: Option<WindowSystemTheme>,
    pub(crate) apply_default_theme: bool,
    pub(crate) font_embolden: f32,
    #[allow(dead_code)]
    pub(crate) mac_os_config: Option<MacOSWindowConfig>,
    pub(crate) win_os_config: Option<WinOSWindowConfig>,
    pub(crate) web_config: Option<WebWindowConfig>,
}

#[cfg(all(feature = "baseview", not(feature = "winit")))]
impl From<WindowConfig> for baseview::WindowOpenOptions {
    fn from(value: WindowConfig) -> Self {
        let sz = value.initial_size();
        let size = baseview::Size::new(sz.width, sz.height);
        baseview::WindowOpenOptions { title: value.title, size: size, scale: baseview::WindowScalePolicy::SystemScaleFactor, gl_config: None }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            size: None,
            min_size: None,
            max_size: None,
            position: None,
            show_titlebar: true,
            transparent: false,
            #[cfg(all(feature = "winit", not(feature = "baseview")))]
            fullscreen: None,
            #[cfg(all(feature = "winit", not(feature = "baseview")))]
            window_icon: None,
            title: std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
                .unwrap_or("Floem Window".to_string()),
            #[cfg(all(feature = "winit", not(feature = "baseview")))]
            enabled_buttons: WindowButtons::all(),
            resizable: true,
            undecorated: false,
            undecorated_shadow: false,
            #[cfg(all(feature = "winit", not(feature = "baseview")))]
            window_level: WindowLevel::Normal,
            theme_override: None,
            apply_default_theme: true,
            font_embolden: if cfg!(target_os = "macos") { 0.2 } else { 0. },
            mac_os_config: None,
            win_os_config: None,
            web_config: None,
        }
    }
}

impl WindowConfig {
    #[cfg(all(feature = "baseview", not(feature = "winit")))]
    fn initial_size(&self) -> Size {
        self.size.unwrap_or_else(||self.min_size.unwrap_or_else(||self.max_size.unwrap_or(Size::new(512., 512.))))
    }

    /// Requests the window to be of specific dimensions.
    ///
    /// If this is not set, some platform-specific dimensions will be used.
    #[inline]
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Requests the window to be of specific min dimensions.
    #[inline]
    pub fn min_size(mut self, size: impl Into<Size>) -> Self {
        self.min_size = Some(size.into());
        self
    }

    /// Requests the window to be of specific max dimensions.
    #[inline]
    pub fn max_size(mut self, size: impl Into<Size>) -> Self {
        self.max_size = Some(size.into());
        self
    }

    /// Sets a desired initial position for the window.
    ///
    /// If this is not set, some platform-specific position will be chosen.
    #[inline]
    pub fn position(mut self, position: Point) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets whether the window should have a title bar.
    ///
    /// The default is `true`.
    #[inline]
    pub fn show_titlebar(mut self, show_titlebar: bool) -> Self {
        self.show_titlebar = show_titlebar;
        self
    }

    /// Sets whether the window should have a border, a title bar, etc.
    ///
    /// The default is `false`.
    #[inline]
    pub fn undecorated(mut self, undecorated: bool) -> Self {
        self.undecorated = undecorated;
        self
    }

    /// Sets whether the window should have background drop shadow when undecorated.
    ///
    /// The default is `false`.
    #[inline]
    pub fn undecorated_shadow(mut self, undecorated_shadow: bool) -> Self {
        self.undecorated_shadow = undecorated_shadow;
        self
    }

    /// Sets whether the background of the window should be transparent.
    ///
    /// The default is `false`.
    #[inline]
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    /// Sets whether the window should be put into fullscreen upon creation.
    ///
    /// The default is `None`.
    #[inline]
    pub fn fullscreen(mut self, fullscreen: Fullscreen) -> Self {
        self.fullscreen = Some(fullscreen);
        self
    }

    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    /// Sets the window icon.
    ///
    /// The default is `None`.
    #[inline]
    pub fn window_icon(mut self, window_icon: Icon) -> Self {
        self.window_icon = Some(window_icon);
        self
    }

    /// Sets the initial title of the window in the title bar.
    ///
    /// The default is `"Floem window"`.
    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    /// Sets the enabled window buttons.
    ///
    /// The default is `WindowButtons::all()`.
    #[inline]
    pub fn enabled_buttons(mut self, enabled_buttons: WindowButtons) -> Self {
        self.enabled_buttons = enabled_buttons;
        self
    }

    /// Sets whether the window is resizable or not.
    ///
    /// The default is `true`.
    #[inline]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    /// Sets the window level.
    ///
    /// This is just a hint to the OS, and the system could ignore it.
    ///
    /// The default is `WindowLevel::Normal`.
    #[inline]
    pub fn window_level(mut self, window_level: WindowLevel) -> Self {
        self.window_level = window_level;
        self
    }

    /// Set a theme override for the window.
    ///
    /// If not provided, the window will follow OS theme.
    #[inline]
    pub fn theme_override(mut self, theme_override: impl Into<WindowSystemTheme>) -> Self {
        // use Impl<Into<WindowSystemTheme>> so old code that uses winit::Theme can build unmodified
        self.theme_override = Some(theme_override.into());
        self
    }

    /// .
    #[inline]
    pub fn apply_default_theme(mut self, apply: bool) -> Self {
        self.apply_default_theme = apply;
        self
    }

    /// Sets the amount by which fonts are emboldened.
    ///
    /// The default is 0.0 except for on macOS where the default is 0.2
    #[inline]
    pub fn font_embolden(mut self, font_embolden: f32) -> Self {
        self.font_embolden = font_embolden;
        self
    }

    /// Set up Mac-OS specific configuration.  The passed closure will only be
    /// called on macOS.
    #[allow(unused_variables, unused_mut)] // build will complain on non-macOS's otherwise
    pub fn with_mac_os_config(
        mut self,
        mut f: impl FnMut(MacOSWindowConfig) -> MacOSWindowConfig,
    ) -> Self {
        #[cfg(target_os = "macos")]
        if let Some(existing_config) = self.mac_os_config {
            self.mac_os_config = Some(f(existing_config))
        } else {
            let new_config = f(MacOSWindowConfig::default());
            self.mac_os_config = Some(new_config);
        }
        self
    }

    /// Set up Windows specific configuration. The passed closure will only be
    /// called on Windows.
    #[allow(unused_variables, unused_mut)] // build will complain on non-Windows platforms otherwise
    pub fn with_win_os_config(
        mut self,
        mut f: impl FnMut(WinOSWindowConfig) -> WinOSWindowConfig,
    ) -> Self {
        #[cfg(target_os = "windows")]
        if let Some(existing_config) = self.win_os_config {
            self.win_os_config = Some(f(existing_config))
        } else {
            let new_config = f(WinOSWindowConfig::default());
            self.win_os_config = Some(new_config);
        }
        self
    }

    /// Set up web specific configuration.
    /// The passed closure will only be called on the web.
    #[allow(unused_variables, unused_mut)] // build will complain on non-web platforms otherwise
    pub fn with_web_config(mut self, f: impl FnOnce(WebWindowConfig) -> WebWindowConfig) -> Self {
        #[cfg(target_arch = "wasm32")]
        if let Some(existing_config) = self.web_config {
            self.web_config = Some(f(existing_config))
        } else {
            let new_config = f(WebWindowConfig {
                canvas_id: String::new(),
            });
            self.web_config = Some(new_config);
        }
        self
    }
}
