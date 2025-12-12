//! For historical reasons, all of these modules are namespaced into `crate::window` but `pub use` statements,
//! and should not be exposed via this namespace.

pub(crate) mod window_config;
pub(crate) mod window_config_mac;
pub(crate) mod window_config_web;
pub(crate) mod window_config_win;
