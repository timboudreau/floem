use crate::{
    action::Timer,
    app_events::UserEvent,
    application::{
        app_handle::ApplicationHandle,
        spi::{AppHandlerCommon, AppHandlerImpl, AppHandlerInternalAPI},
    },
    context::PaintState,
    profiler::ProfileEvent,
    window::{WindowConfig, WindowCreation},
    windows::window_handle::WindowHandle,
    AppConfig, View,
};
use adapters::WindowSystemTheme;
use dpi::PhysicalPosition;
use event::FileDragEvent::{self, DragDropped};
use peniko::kurbo::{Point, Size};
use std::collections::HashMap;
use ui_events_winit::WindowEventTranslation;
use windowing::{
    internal_api::WindowingBackend as _, internal_api::WindowingSystem,
    public_api::WindowIdentifier,
};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
    window::Theme,
};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;

impl AppHandlerInternalAPI for ApplicationHandle {
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

    fn handle_window_creation(
        &mut self,
        creation: WindowCreation,
        event_loop: &dyn ActiveEventLoop,
    ) {
        self.new_window(
            event_loop,
            creation.view_fn,
            self.config
                .global_theme_override
                .map(WindowSystemTheme::from),
            creation.config.unwrap_or_default(),
        )
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

        #[cfg(all(target_arch = "wasm32", feature = "winit"))]
        {
            window_attributes =
                super::wasm_window_specialiation::apply_wasm_specialization(window_attributes);
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
            // Mac OS has more nuanced control of the titlebar, independent of the
            // window being undecorated; for other OS's we must simply disable decorations
            // (which on some platforms may also disable drag/resize behavior).
            window_attributes = window_attributes.with_decorations(false);
        }

        #[cfg(target_os = "windows")]
        {
            window_attributes = super::os_windows::apply_windows_specialization(
                window_attributes,
                undecorated_shadow,
                win_os_config,
            );
        }

        #[cfg(all(target_os = "macos", feature = "winit"))]
        {
            window_attributes = super::os_mac::apply_mac_os_attributes(
                window_attributes,
                show_titlebar,
                undecorated,
                &mac_os_config,
            );
        }

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => window,
            Err(err) => {
                // At least log it - do we have a better way of reporting errors?
                eprintln!("Failed to create window: {err}");
                return;
            }
        };

        #[cfg(target_os = "macos")]
        {
            // This one we do without conditioning it on `winit`, but whether it will do anything useful
            // is TBD.
            super::os_mac::mac_os_post_window_creation_config(&window, &mac_os_config);
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

    fn remove_timer(
        &mut self,
        timer: &crate::action::TimerToken,
        event_loop: &Self::WindowingSystemEventLoop,
    ) {
        self.timers.remove(timer);
        if self.timers.is_empty() {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn handle_user_event(&mut self, event_loop: &Self::WindowingSystemEventLoop, event: UserEvent) {
        self.internal_handle_user_event(event_loop, event);
    }

    fn handle_updates_for_all_windows(&mut self) {
        for (window_id, handle) in self.window_handles.iter_mut() {
            handle.process_update();
            while WindowingSystem::process_window_updates(window_id) {}
        }
    }

    fn handle_timer(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.internal_handle_timer(event_loop);
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
}

impl AppHandlerImpl for ApplicationHandle {
    fn close_window(&mut self, window_id: WindowIdentifier, event_loop: &dyn ActiveEventLoop) {
        if let Some(handle) = self.window_handles.get_mut(&window_id) {
            handle.destroy();
        }
        self.window_handles.remove(&window_id);
        if self.window_handles.is_empty() && self.config.exit_on_close {
            self.handle_exit(event_loop);
        }
    }

    fn request_timer(&mut self, timer: Timer, event_loop: &dyn ActiveEventLoop) {
        self.timers.insert(timer.token, timer);
        self.fire_timer(event_loop);
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

    fn handle_gpu_resource_update(&mut self, window_id: WindowIdentifier) {
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
}
