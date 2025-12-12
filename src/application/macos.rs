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
#[cfg(target_os = "macos")]
pub(super) fn setup_traffic_light_constraints_all_pixels(
    view_handle: &raw_window_handle::AppKitWindowHandle,
    leading_pixels: f64,
    top_pixels: f64,
    button_spacing_pixels: f64,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use {
        objc2_app_kit::{NSLayoutAttribute, NSLayoutConstraint, NSLayoutRelation, NSWindowButton},
        objc2_foundation::NSArray,
    };

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
