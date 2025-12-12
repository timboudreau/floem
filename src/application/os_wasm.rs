//! New window customizations for WASM.
use wgpu::web_sys;
use wgpu::web_sys::wasm_bindgen::JsCast;
use winit::platform::web::WindowAttributesExtWeb;

pub(super) fn apply_wasm_specialization(
    mut window_attributes: winit::window::WindowAttributes,
) -> winit::window::WindowAttributes {
    let parent_id = web_config.expect("Specify an id for the canvas.").canvas_id;
    let doc = web_sys::window()
        .and_then(|win| win.document())
        .expect("Couldn't get document.");
    let canvas = doc
        .get_element_by_id(&parent_id)
        .expect("Couldn't get canvas by supplied id.");
    let canvas = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("Element behind supplied id is not a canvas.");

    if let Some(size) = logical_size {
        canvas.set_width(size.width as u32);
        canvas.set_height(size.height as u32);
    }
    window_attributes.with_canvas(Some(canvas))
}
