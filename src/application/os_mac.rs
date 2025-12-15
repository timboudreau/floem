//! New window customizations for MacOS.
#![cfg(target_os = "macos")]

#[cfg(feature = "winit")]
use crate::window::MacOSWindowConfig;
use {
    objc2_app_kit::{NSLayoutAttribute, NSLayoutConstraint, NSLayoutRelation, NSWindowButton},
    objc2_foundation::NSArray,
};

#[cfg(feature = "winit")]
pub(super) fn apply_mac_os_attributes(
    window_attributes: winit::window::WindowAttributes,
    show_titlebar: bool,
    undecorated: bool,
    mac_os_config: &Option<MacOSWindowConfig>,
) -> winit::window::WindowAttributes {
    // Moving this out of a giant stanza in `AppHandle.new_window()`.

    // This relies on winit specific types, and for baseview, since the window is ordinarily created
    // by the host application (in Logic Pro or Garage Band, the window is not even created by the same
    // *process* as the plugin), there is no equivalent there.

    let mut mac_attrs = winit::platform::macos::WindowAttributesMacOS::default();

    if !show_titlebar {
        mac_attrs = mac_attrs
            .with_movable_by_window_background(false)
            .with_title_hidden(true)
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true);
        // .with_traffic_lights_offset(11.0, 16.0);
    }
    if undecorated {
        // A palette-style window that will only obtain window focus but
        // not actually propagate the first mouse click it receives is
        // very unlikely to be expected behavior - these typically are
        // used for something that offers a quick choice and are closed
        // in a single pointer gesture.
        mac_attrs = mac_attrs.with_accepts_first_mouse(true);
    }
    if let Some(mac) = mac_os_config {
        if let Some(val) = mac.movable_by_window_background {
            mac_attrs = mac_attrs.with_movable_by_window_background(val);
        }
        if let Some(val) = mac.titlebar_transparent {
            mac_attrs = mac_attrs.with_titlebar_transparent(val);
        }
        if let Some(val) = mac.titlebar_hidden {
            mac_attrs = mac_attrs.with_titlebar_hidden(val);
        }
        if let Some(val) = mac.title_hidden {
            mac_attrs = mac_attrs.with_title_hidden(val);
        }
        if let Some(val) = mac.full_size_content_view {
            mac_attrs = mac_attrs.with_fullsize_content_view(val);
        }
        if let Some(val) = mac.unified_titlebar {
            mac_attrs = mac_attrs.with_unified_titlebar(val);
        }
        if let Some(val) = mac.movable {
            mac_attrs = mac_attrs.with_movable_by_window_background(val);
        }
        if let Some(val) = mac.accepts_first_mouse {
            mac_attrs = mac_attrs.with_accepts_first_mouse(val);
        }
        if let Some(val) = mac.option_as_alt {
            mac_attrs = mac_attrs.with_option_as_alt(val.into());
        }
        if let Some(title) = &mac.tabbing_identifier {
            mac_attrs = mac_attrs.with_tabbing_identifier(title.as_str());
        }
        if let Some(disallow_hidpi) = mac.disallow_high_dpi {
            mac_attrs = mac_attrs.with_disallow_hidpi(disallow_hidpi);
        }
        if let Some(shadow) = mac.has_shadow {
            mac_attrs = mac_attrs.with_has_shadow(shadow);
        }
        if let Some(hide) = mac.titlebar_buttons_hidden {
            mac_attrs = mac_attrs.with_titlebar_buttons_hidden(hide)
        }
        // if let Some(panel) = mac.panel {
        //     window_attributes = window_attributes.with_panel(panel)
        // }
    }
    // Note, previously, this call was inside the if/then above, and would never run unless a mac os
    // config was also set (undecorated and !show_titlebar would not work if there wasn't also a mac_attrs present).
    // That was almost certainly a bug.
    window_attributes.with_platform_attributes(Box::new(mac_attrs))
}

#[cfg(feature = "winit")]
pub(super) fn mac_os_post_window_creation_config(
    window: &Box<dyn winit::window::Window>,
    mac_os_config: &Option<MacOSWindowConfig>,
) {
    if let Some(mac) = &mac_os_config {
        if let Some((x, y)) = mac.traffic_lights_offset {
            use raw_window_handle::HasWindowHandle;

            if let Ok(wh) = window.window_handle() {
                use raw_window_handle::RawWindowHandle;

                if let RawWindowHandle::AppKit(app_kit) = wh.as_raw() {
                    let _ = super::os_mac::setup_traffic_light_constraints_all_pixels(
                        &app_kit, x, y, 6.,
                    );
                }
            }
        }
    }
}

// Leaving the below as available on non-winit windowing systems, though it may not
// turn out to have utility on hosted windows.

/// Sets up traffic light button constraints with precise pixel positioning.
///
/// # Parameters
/// - `leading_pixels`: Distance from left edge of title bar to close button
///   (typically 10.0 for standard macOS positioning)
/// - `top_pixels`: Distance from top edge of title bar to **top edge** of
///   buttons
/// - `button_spacing_pixels`: Spacing between traffic light buttons (typically
///   6.0 for native macOS appearance)
///
/// # Calculating `top_pixels` for vertical centering
/// Traffic light buttons are typically 13pt tall, so to center them:
/// `top_pixels = (top_bar_height - 13.0) / 2.0`
///
/// # Example for centering in a 30pt top bar
/// ```rust,ignore
/// // Standard horizontal position (10pt), centered vertically (8.5pt from top), standard spacing (6pt)
/// setup_traffic_light_constraints_all_pixels(view_handle, 10.0, 8.5, 6.0)?;
/// ```
///
/// # Common values
/// - Standard positioning: `(10.0, 8.0, 6.0)`
/// - Centered in 30pt bar: `(10.0, 8.5, 6.0)`
/// - Centered in 40pt bar: `(10.0, 13.5, 6.0)`
/// - Centered in 50pt bar: `(10.0, 18.5, 6.0)`
#[cfg_attr(feature = "baseview", allow(unused))]
pub(super) fn setup_traffic_light_constraints_all_pixels(
    view_handle: &raw_window_handle::AppKitWindowHandle,
    leading_pixels: f64,
    top_pixels: f64,
    button_spacing_pixels: f64,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let ns_view = view_handle.ns_view.cast::<objc2_app_kit::NSView>();
    let ns_view = unsafe { &*ns_view.as_ptr() };
    let window = ns_view
        .window()
        .ok_or("View must be attached to a window")?;

    let close_button = window.standardWindowButton(NSWindowButton::CloseButton);
    let miniaturize_button = window.standardWindowButton(NSWindowButton::MiniaturizeButton);
    let zoom_button = window.standardWindowButton(NSWindowButton::ZoomButton);
    let title_bar_view = close_button
        .as_ref()
        .and_then(|button| unsafe { button.superview() })
        .ok_or("Could not find title bar container view")?;

    unsafe {
        // Set up close button with exact pixel positioning
        if let Some(close_btn) = &close_button {
            close_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

            let leading = NSLayoutConstraint::constraintWithItem_attribute_relatedBy_toItem_attribute_multiplier_constant(
                close_btn,
                NSLayoutAttribute::Leading,
                NSLayoutRelation::Equal,
                Some(&title_bar_view),
                NSLayoutAttribute::Leading,
                1.0,
                leading_pixels,
            );

            let top = NSLayoutConstraint::constraintWithItem_attribute_relatedBy_toItem_attribute_multiplier_constant(
                close_btn,
                NSLayoutAttribute::Top,
                NSLayoutRelation::Equal,
                Some(&title_bar_view),
                NSLayoutAttribute::Top,
                1.0,
                top_pixels,
            );

            title_bar_view.addConstraints(&NSArray::from_slice(&[&*leading, &*top]));
        }

        // Set up other buttons with custom spacing
        if let (Some(mini_btn), Some(close_btn)) = (&miniaturize_button, &close_button) {
            mini_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

            let leading = NSLayoutConstraint::constraintWithItem_attribute_relatedBy_toItem_attribute_multiplier_constant(
                mini_btn,
                NSLayoutAttribute::Leading,
                NSLayoutRelation::Equal,
                Some(close_btn),
                NSLayoutAttribute::Trailing,
                1.0,
                button_spacing_pixels,
            );

            let center_y = NSLayoutConstraint::constraintWithItem_attribute_relatedBy_toItem_attribute_multiplier_constant(
                mini_btn,
                NSLayoutAttribute::CenterY,
                NSLayoutRelation::Equal,
                Some(close_btn),
                NSLayoutAttribute::CenterY,
                1.0,
                0.0,
            );

            title_bar_view.addConstraints(&NSArray::from_slice(&[&*leading, &*center_y]));
        }

        if let (Some(zoom_btn), Some(mini_btn)) = (&zoom_button, &miniaturize_button) {
            zoom_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

            let leading = NSLayoutConstraint::constraintWithItem_attribute_relatedBy_toItem_attribute_multiplier_constant(
                zoom_btn,
                NSLayoutAttribute::Leading,
                NSLayoutRelation::Equal,
                Some(mini_btn),
                NSLayoutAttribute::Trailing,
                1.0,
                button_spacing_pixels,
            );

            let center_y = NSLayoutConstraint::constraintWithItem_attribute_relatedBy_toItem_attribute_multiplier_constant(
                zoom_btn,
                NSLayoutAttribute::CenterY,
                NSLayoutRelation::Equal,
                Some(mini_btn),
                NSLayoutAttribute::CenterY,
                1.0,
                0.0,
            );

            title_bar_view.addConstraints(&NSArray::from_slice(&[&*leading, &*center_y]));
        }
    }

    Ok(())
}
