use adapters::WindowSystemTheme;
use dpi::PhysicalPosition;
use floem_renderer::gpu_resources::GpuResources;
use muda::MenuId;
use windowing::internal_api::{WindowingBackend, WindowingSystem};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(all(feature = "winit", not(feature = "baseview")))]
use ui_events_winit::WindowEventTranslation;

#[cfg(all(feature = "baseview", not(feature = "winit")))]
use ui_events_baseview::WindowEventTranslation;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[cfg(target_arch = "wasm32")]
use wgpu::web_sys;

use floem_reactive::{SignalUpdate, WriteSignal};
use peniko::kurbo::{Point, Size};
use std::{collections::HashMap, rc::Rc};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
    window::Theme,
};

use crate::{app_config::AppConfig, application::app_base::AppHandlerInternalAPI, window::{WindowCreation}};
use super::app_base::AppHandlerImpl;

use crate::{
    AppEvent, WindowIdentifier,
    action::{Timer, TimerToken},
    app_events::AppEventCallback,
    app_events::UserEvent,
    context::PaintState,
    event::FileDragEvent::{self, DragDropped},
    ext_event::EXT_EVENT_HANDLER,
    inspector::Capture,
    profiler::{Profile, ProfileEvent},
    view::View,
    window::WindowConfig,
    windows::window_handle::WindowHandle,
};

pub type ApplicationHandle = WinitApplicationHandle;

pub(crate) struct WinitApplicationHandle {
    window_handles: HashMap<WindowIdentifier, WindowHandle>,
    timers: HashMap<TimerToken, Timer>,
    pub(crate) event_listener: Option<Box<AppEventCallback>>,
    gpu_resources: Option<GpuResources>,
    pub(crate) config: AppConfig,
}

impl AppHandlerInternalAPI for WinitApplicationHandle {
    type WindowingSystemEventLoop = dyn ActiveEventLoop;
    type WindowingSystemWindowEvent = WindowEvent;

    fn new(config: AppConfig) -> Self {
        Self {
            window_handles: HashMap::new(),
            timers: HashMap::new(),
            event_listener: None,
            gpu_resources: None,
            config,
        }
    }

    fn new_window(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        view_fn: Box<dyn FnOnce(WindowIdentifier) -> Box<dyn View>>,
        override_theme: Option<WindowSystemTheme>,
        #[allow(unused_variables)] WindowConfig {
            size,
            min_size,
            max_size,
            position,
            show_titlebar,
            transparent,
            fullscreen,
            window_icon,
            title,
            enabled_buttons,
            resizable,
            undecorated,
            undecorated_shadow,
            window_level,
            theme_override,
            apply_default_theme,
            mac_os_config,
            win_os_config,
            web_config,
            font_embolden,
        }: WindowConfig,
    ) {
        let logical_size = size.map(|size| LogicalSize::new(size.width, size.height));
        let logical_min_size = min_size.map(|size| LogicalSize::new(size.width, size.height));
        let logical_max_size = max_size.map(|size| LogicalSize::new(size.width, size.height));

        #[cfg(target_os = "macos")]
        let mut mac_attrs = winit::platform::macos::WindowAttributesMacOS::default();

        let mut window_attributes = winit::window::WindowAttributes::default()
            .with_visible(false)
            .with_title(title)
            .with_decorations(!undecorated)
            .with_transparent(transparent)
            .with_fullscreen(fullscreen)
            .with_window_level(window_level)
            .with_window_icon(window_icon)
            .with_resizable(resizable)
            // .with_theme(theme_override)
            .with_enabled_buttons(enabled_buttons);
        if theme_override.is_none() {
            window_attributes = window_attributes.with_theme(override_theme.map(Theme::from));
        } else {
            window_attributes = window_attributes.with_theme(theme_override.map(Theme::from));
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wgpu::web_sys::wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWeb;

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

            window_attributes = window_attributes.with_canvas(Some(canvas));
        };

        if let Some(Point { x, y }) = position {
            window_attributes = window_attributes.with_position(LogicalPosition::new(x, y));
        }

        if let Some(logical_size) = logical_size {
            window_attributes = window_attributes.with_surface_size(logical_size);
        }
        if let Some(logical_min_size) = logical_min_size {
            window_attributes = window_attributes.with_min_surface_size(logical_min_size);
        }
        if let Some(logical_max_size) = logical_max_size {
            window_attributes = window_attributes.with_max_surface_size(logical_max_size);
        }

        #[cfg(not(target_os = "macos"))]
        if !show_titlebar {
            window_attributes = window_attributes.with_decorations(false);
        }

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowAttributesWindows;
            let mut win =
                WindowAttributesWindows::default().with_undecorated_shadow(undecorated_shadow);
            if let Some(cfg) = win_os_config {
                use crate::window::convert_to_win;
                win = win
                    .with_title_background_color(convert_to_win(cfg.set_title_background_color))
                    .with_border_color(convert_to_win(cfg.set_border_color))
                    .with_skip_taskbar(cfg.set_skip_taskbar)
                    .with_corner_preference(cfg.corner_preference.into())
                    .with_system_backdrop(cfg.set_system_backdrop.into())
                    .with_title_text_color(
                        convert_to_win(cfg.set_title_text_color).unwrap_or_default(),
                    );
            }
            window_attributes = window_attributes.with_platform_attributes(Box::new(win));
        }

        #[cfg(target_os = "macos")]
        if !show_titlebar {
            mac_attrs = mac_attrs
                .with_movable_by_window_background(false)
                .with_title_hidden(true)
                .with_titlebar_transparent(true)
                .with_fullsize_content_view(true);
            // .with_traffic_lights_offset(11.0, 16.0);
        }

        #[cfg(target_os = "macos")]
        if undecorated {
            // A palette-style window that will only obtain window focus but
            // not actually propagate the first mouse click it receives is
            // very unlikely to be expected behavior - these typically are
            // used for something that offers a quick choice and are closed
            // in a single pointer gesture.
            mac_attrs = mac_attrs.with_accepts_first_mouse(true);
        }

        #[cfg(target_os = "macos")]
        if let Some(mac) = &mac_os_config {
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
            window_attributes = window_attributes.with_platform_attributes(Box::new(mac_attrs));
        }

        let Ok(window) = event_loop.create_window(window_attributes) else {
            return;
        };
        #[cfg(target_os = "macos")]
        if let Some(mac) = &mac_os_config {
            if let Some((x, y)) = mac.traffic_lights_offset {
                use raw_window_handle::HasWindowHandle;

                if let Ok(wh) = window.window_handle() {
                    use raw_window_handle::RawWindowHandle;

                    if let RawWindowHandle::AppKit(app_kit) = wh.as_raw() {
                        let _ = super::macos::setup_traffic_light_constraints_all_pixels(&app_kit, x, y, 6.);
                    }
                }
            }
        }
        let window_id = window.id();
        let window_handle = WindowHandle::new(
            window,
            self.gpu_resources.clone(),
            self.config.wgpu_features,
            view_fn,
            transparent,
            apply_default_theme,
            font_embolden,
        );
        self.window_handles.insert(window_id.into(), window_handle);
    }

    fn handle_user_event(&mut self, event_loop: &Self::WindowingSystemEventLoop, event: UserEvent) {
        self.internal_handle_user_event(event_loop, event);
    }

    fn idle(&mut self) {
        let ext_events = { std::mem::take(&mut *EXT_EVENT_HANDLER.queue.lock()) };

        for trigger in ext_events {
            trigger.notify();
        }

        self.handle_updates_for_all_windows();
    }

    fn handle_updates_for_all_windows(&mut self) {
        for (window_id, handle) in self.window_handles.iter_mut() {
            handle.process_update();
            while WindowingSystem::process_window_updates(window_id) {}
        }
    }

    fn handle_timer(&mut self, event_loop: &dyn ActiveEventLoop) {
        let now = Instant::now();
        let tokens: Vec<TimerToken> = self
            .timers
            .iter()
            .filter_map(|(token, timer)| {
                if timer.deadline <= now {
                    Some(*token)
                } else {
                    None
                }
            })
            .collect();
        if !tokens.is_empty() {
            for token in tokens {
                if let Some(timer) = self.timers.remove(&token) {
                    (timer.action)(token);
                }
            }
            self.handle_updates_for_all_windows();
        }
        self.fire_timer(event_loop);
    }

    fn handle_window_event(
        &mut self,
        window_id: WindowIdentifier,
        event: WindowEvent,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let window_handle = match self.window_handles.get_mut(&window_id) {
            Some(window_handle) => window_handle,
            None => return,
        };

        let start = window_handle.profile.is_some().then(|| {
            let name = match event {
                WindowEvent::ActivationTokenDone { .. } => "ActivationTokenDone",
                WindowEvent::SurfaceResized(..) => "Resized",
                WindowEvent::Moved(..) => "Moved",
                WindowEvent::CloseRequested => "CloseRequested",
                WindowEvent::Destroyed => "Destroyed",
                WindowEvent::Focused(..) => "Focused",
                WindowEvent::KeyboardInput { .. } => "KeyboardInput",
                WindowEvent::ModifiersChanged(..) => "ModifiersChanged",
                WindowEvent::Ime(..) => "Ime",
                WindowEvent::PointerMoved { .. } => "PointerMoved",
                WindowEvent::PointerEntered { .. } => "PointerEntered",
                WindowEvent::PointerLeft { .. } => "PointerLeft",
                WindowEvent::MouseWheel { .. } => "MouseWheel",
                WindowEvent::PointerButton { .. } => "PointerButton",
                WindowEvent::TouchpadPressure { .. } => "TouchpadPressure",
                WindowEvent::ScaleFactorChanged { .. } => "ScaleFactorChanged",
                WindowEvent::ThemeChanged(..) => "ThemeChanged",
                WindowEvent::Occluded(..) => "Occluded",
                WindowEvent::RedrawRequested => "RedrawRequested",
                WindowEvent::PinchGesture { .. } => "PinchGesture",
                WindowEvent::PanGesture { .. } => "PanGesture",
                WindowEvent::DoubleTapGesture { .. } => "DoubleTapGesture",
                WindowEvent::RotationGesture { .. } => "RotationGesture",
                WindowEvent::DragDropped { .. } => "DroppedFile",
                WindowEvent::DragEntered { .. } => "DragEntered",
                WindowEvent::DragLeft { .. } => "DragLeft",
                WindowEvent::DragMoved { .. } => "DragMoved",
            };
            (
                name,
                Instant::now(),
                matches!(event, WindowEvent::RedrawRequested),
            )
        });

        match window_handle
            .event_reducer
            .reduce(window_handle.scale, &event)
        {
            Some(WindowEventTranslation::Keyboard(ke)) => {
                if let WindowEvent::KeyboardInput { is_synthetic, .. } = event {
                    if !is_synthetic {
                        window_handle.key_event(ke)
                    }
                }
            }
            Some(WindowEventTranslation::Pointer(pe)) => {
                window_handle.pointer_event(pe);
            }
            None => {}
        }

        match event {
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::SurfaceResized(size) => {
                let size: LogicalSize<f64> = size.to_logical(window_handle.scale);
                let size = Size::new(size.width, size.height);
                window_handle.size(size);
            }
            WindowEvent::Moved(position) => {
                let position: LogicalPosition<f64> = position.to_logical(window_handle.scale);
                let point = Point::new(position.x, position.y);
                window_handle.position(point);
            }
            WindowEvent::CloseRequested => {
                self.close_window(window_id, event_loop);
            }
            WindowEvent::Destroyed => {
                self.close_window(window_id, event_loop);
            }
            WindowEvent::DragDropped { paths, position } => {
                window_handle.file_drag_event(DragDropped {
                    paths,
                    position: PhysicalPosition::new(position.x, position.y),
                    scale_factor: window_handle.scale,
                });
            }
            WindowEvent::DragEntered { paths, position } => {
                window_handle.file_drag_event(FileDragEvent::DragEntered {
                    paths,
                    position: PhysicalPosition::new(position.x, position.y),
                    scale_factor: window_handle.scale,
                });
            }
            WindowEvent::DragMoved { position } => {
                window_handle.file_drag_event(FileDragEvent::DragMoved {
                    position: PhysicalPosition::new(position.x, position.y),
                    scale_factor: window_handle.scale,
                });
            }
            WindowEvent::DragLeft { position } => {
                let pos = position.map(|p| PhysicalPosition::new(p.x, p.y));
                window_handle.file_drag_event(FileDragEvent::DragLeft {
                    position: pos,
                    scale_factor: window_handle.scale,
                });
            }
            WindowEvent::Focused(focused) => {
                window_handle.focused(focused);
            }
            WindowEvent::KeyboardInput { .. } => {
                // already handled by the ui-events reducer
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                window_handle.modifiers_changed(
                    ui_events_winit::keyboard::from_winit_modifier_state(modifiers.state()),
                );
            }
            WindowEvent::Ime(ime) => {
                window_handle.ime(ime);
            }
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::PinchGesture {
                delta: _, phase: _, ..
            } => {}
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                window_handle.scale(scale_factor);
            }
            WindowEvent::ThemeChanged(theme) => {
                window_handle.set_theme(Some(theme.into()), true);
            }
            WindowEvent::Occluded(_) => {}
            WindowEvent::RedrawRequested => {
                window_handle.render_frame(self.gpu_resources.clone());
            }
            WindowEvent::PanGesture { .. } => {}
            WindowEvent::DoubleTapGesture { .. } => {}
            WindowEvent::RotationGesture { .. } => {}
            WindowEvent::PointerMoved { .. } => {
                //already handled by the ui-events reducer
            }
            WindowEvent::PointerEntered { .. } => {
                //already handled by the ui-events reducer
            }
            WindowEvent::PointerLeft { .. } => {
                //already handled by the ui-events reducer
            }
            WindowEvent::PointerButton { .. } => {
                //already handled by the ui-events reducer
            }
        }

        if let Some((name, start, new_frame)) = start {
            let end = Instant::now();

            if let Some(window_handle) = self.window_handle_for_window_id(&window_id) {
                let profile = window_handle.profile.as_mut().unwrap();

                profile
                    .current
                    .events
                    .push(ProfileEvent { start, end, name });

                if new_frame {
                    profile.next_frame();
                }
            }
        }
        self.handle_updates_for_all_windows();
    }

    fn window_handle_for_window_id(&mut self, window_id : &WindowIdentifier) -> Option<&mut WindowHandle> {
        self.window_handles.get_mut(window_id)
    }
}

impl super::app_base::AppHandlerImpl for WinitApplicationHandle {


    fn handle_profile(&mut self, window_id: WindowIdentifier, end_profile: Option<WriteSignal<Option<Rc<Profile>>>>) {
        let handle = self.window_handles.get_mut(&window_id);
        if let Some(handle) = handle {
            if let Some(profile) = end_profile {
                profile.set(handle.profile.take().map(|mut profile| {
                    profile.next_frame();
                    Rc::new(profile)
                }));
            } else {
                handle.profile = Some(Profile::default());
            }
        }
    }

    fn handle_window_creation(&mut self, creation: WindowCreation, event_loop: &dyn ActiveEventLoop) {
        self.new_window(
            event_loop,
            creation.view_fn,
            self.config.global_theme_override.map(WindowSystemTheme::from),
            creation.config.unwrap_or_default(),
        )
    }

    fn handle_theme_change(&mut self, theme : WindowSystemTheme) {
        self.config.global_theme_override = Some(theme);
        for window_handle in self.window_handles.values_mut() {
            window_handle.window_state.light_dark_theme = theme;
            window_handle.set_theme(Some(theme), false);
        }
    }

    fn handle_menu_action(&mut self, action_id : MenuId) {
        for (_, handle) in self.window_handles.iter_mut() {
            if handle.window_state.context_menu.contains_key(&action_id)
                || handle.window_menu_actions.contains_key(&action_id)
            {
                handle.menu_action(&action_id);
                break;
            }
        }
    }

    fn close_window(&mut self, window_id: WindowIdentifier, event_loop: &dyn ActiveEventLoop) {
        if let Some(handle) = self.window_handles.get_mut(&window_id) {
            handle.destroy();
        }
        self.window_handles.remove(&window_id);
        if self.window_handles.is_empty() && self.config.exit_on_close {
            event_loop.exit();
        }
    }

    fn capture_window(&mut self, window_id: WindowIdentifier) -> Option<Capture> {
        self.window_handles
            .get_mut(&window_id)
            .map(|handle| handle.capture(self.gpu_resources.clone()))
    }

    fn request_timer(&mut self, timer: Timer, event_loop: &dyn ActiveEventLoop) {
        self.timers.insert(timer.token, timer);
        self.fire_timer(event_loop);
    }

    fn remove_timer(&mut self, timer: &TimerToken, event_loop: &dyn ActiveEventLoop) {
        self.timers.remove(timer);
        if self.timers.is_empty() {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn fire_timer(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.timers.is_empty() {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        let deadline = self.timers.values().map(|timer| timer.deadline).min();
        if let Some(deadline) = deadline {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
        }
    }

    fn handle_gpu_resource_update(&mut self, window_id : WindowIdentifier) {
        let handle = self.window_handles.get_mut(&window_id).unwrap();
        if let PaintState::PendingGpuResources {
            window,
            rx,
            font_embolden,
            renderer,
        } = &handle.paint_state
        {
            let (gpu_resources, surface) = rx.recv().unwrap().unwrap();
            let renderer = crate::renderer::Renderer::new(
                window.clone(),
                gpu_resources.clone(),
                surface,
                renderer.scale(),
                renderer.size(),
                *font_embolden,
            );
            self.gpu_resources = Some(gpu_resources);
            handle.paint_state = PaintState::Initialized { renderer };
            handle.init_renderer(self.gpu_resources.clone());
        } else {
            panic!("Sent a gpu resource update after it had already been initialized");
        }
    }

    fn handle_exit(&mut self, event_loop: &dyn ActiveEventLoop) {
        event_loop.exit();
    }

    fn publish_event(&mut self, event : AppEvent) {
        if let Some(action) = self.event_listener.as_ref() {
            action(event);
        }
    }

}
