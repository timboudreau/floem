
/// Web specific window (canvas) configuration properties, accessible via
/// `WindowConfig::with_web_config( WebWindowConfig )`.
#[derive(Default, Debug, Clone)]
pub struct WebWindowConfig {
    /// The id of the HTML canvas element that floem should render to.
    pub(crate) canvas_id: String,
}

impl WebWindowConfig {
    /// Specify the id of the HTML canvas element that floem should render to.
    pub fn canvas_id(mut self, val: impl Into<String>) -> Self {
        self.canvas_id = val.into();
        self
    }
}
