use adapters::WindowSystemTheme;

#[derive(Debug)]
pub struct AppConfig {
    pub(crate) exit_on_close: bool,
    pub(crate) wgpu_features: wgpu::Features,
    pub(crate) global_theme_override: Option<WindowSystemTheme>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            exit_on_close: !cfg!(target_os = "macos"),
            wgpu_features: wgpu::Features::default(),
            global_theme_override: None,
        }
    }
}

impl AppConfig {
    /// Sets whether the application should exit when the last window is closed.
    #[inline]
    pub fn exit_on_close(mut self, exit_on_close: bool) -> Self {
        self.exit_on_close = exit_on_close;
        self
    }

    /// Sets the WGPU features to be used by the application.
    #[inline]
    pub fn wgpu_features(mut self, features: wgpu::Features) -> Self {
        self.wgpu_features = features;
        self
    }

    /// Sets the global theme.
    #[inline]
    pub fn set_global_theme(mut self, theme: WindowSystemTheme) -> Self {
        self.global_theme_override = Some(theme);
        self
    }
}
