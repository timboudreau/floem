//! A place for a few adapter types where we would otherwise need to expose
//! `winit` or `baseview` types.
//!
//! We should only add to this if there is a real need to eliminate a direct dependency
//! on winit - otherwise we risk creating something like Java's lowest-common-denominator
//! AWT.

#[cfg(feature = "winit")]
use winit::window::{ResizeDirection, Theme};

/// A proxy for the windowing framework's representation of the system light/dark/whatever
/// theme, if one is supported.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum WindowSystemTheme {
    Dark,
    #[default]
    Light,
}

impl WindowSystemTheme {

    /// Get the inverse, light for dark, dark for light.
    pub fn opposite(&self) -> Self {
        match self {
            WindowSystemTheme::Dark => WindowSystemTheme::Light,
            WindowSystemTheme::Light => WindowSystemTheme::Dark,
        }
    }

    /// Determine if the theme is dark.
    pub fn is_dark(&self) -> bool {
        matches![self, Self::Dark]
    }

    /// Determine if the theme is light.
    pub fn is_light(&self) -> bool {
        matches![self, Self::Light]
    }
}

// Pending - there is no baseview equivalent, but there may be library options to detect the
// theme cross-platform.

#[cfg(feature = "winit")]
impl From<Theme> for WindowSystemTheme {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Light => Self::Light,
            Theme::Dark => Self::Dark,
        }
    }
}

#[cfg(feature = "winit")]
impl From<WindowSystemTheme> for Theme {
    fn from(value: WindowSystemTheme) -> Self {
        match value {
            WindowSystemTheme::Light => Self::Light,
            WindowSystemTheme::Dark => Self::Dark,
        }
    }
}

/// A proxy for the underlying windowing framework's abstraction for the direction in which
/// a window is being resized, if the framework supports that.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum WindowResizeDirection {
    East,
    North,
    NorthEast,
    NorthWest,
    South,
    #[default]
    SouthEast,
    SouthWest,
    West,
}

#[cfg(feature = "winit")]
impl From<ResizeDirection> for WindowResizeDirection {
    fn from(direction: ResizeDirection) -> Self {
        match direction {
            ResizeDirection::East => Self::East,
            ResizeDirection::North => Self::North,
            ResizeDirection::NorthEast => Self::NorthEast,
            ResizeDirection::NorthWest => Self::NorthWest,
            ResizeDirection::South => Self::South,
            ResizeDirection::SouthEast => Self::SouthEast,
            ResizeDirection::SouthWest => Self::SouthWest,
            ResizeDirection::West => Self::West,
        }
    }
}

#[cfg(feature = "winit")]
impl From<WindowResizeDirection> for ResizeDirection {
    fn from(direction: WindowResizeDirection) -> Self {
        match direction {
            WindowResizeDirection::East => Self::East,
            WindowResizeDirection::North => Self::North,
            WindowResizeDirection::NorthEast => Self::NorthEast,
            WindowResizeDirection::NorthWest => Self::NorthWest,
            WindowResizeDirection::South => Self::South,
            WindowResizeDirection::SouthEast => Self::SouthEast,
            WindowResizeDirection::SouthWest => Self::SouthWest,
            WindowResizeDirection::West => Self::West,
        }
    }
}
