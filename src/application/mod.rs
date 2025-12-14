pub(crate) mod app_handle;
pub(crate) mod spi;

#[cfg(all(feature = "winit", not(feature = "baseview")))]
mod app_handle_winit;

// #[cfg(all(feature = "baseview", not(feature = "winit")))]
#[cfg(feature = "baseview")]
mod app_handle_baseview;

#[cfg(feature = "baseview")]
mod baseview_hacks;

// OS-specific window configuration, platform-dependent:

// os_mac contains some non-wasm specific code, so leave os/feature constraints inside it
mod os_mac;
#[cfg(all(target_arch = "wasm32", feature = "winit"))]
mod os_wasm;
#[cfg(all(target_os = "windows", feature = "winit"))]
mod os_windows;
