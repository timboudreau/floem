use crate::{
    action::{Timer, TimerToken},
    app_events::AppEventCallback,
    inspector::Capture,
    profiler::Profile,
    windows::window_handle::WindowHandle,
    AppEvent, WindowIdentifier,
};
use crate::app_config::AppConfig;
use super::spi::AppHandlerCommon;
use adapters::WindowSystemTheme;
use floem_reactive::{SignalUpdate, WriteSignal};
use floem_renderer::gpu_resources::GpuResources;
use muda::MenuId;
use std::{collections::HashMap, rc::Rc};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

/// Manages an application's configuration, windows, timers, event callbacks and GPU resources.
pub(crate) struct ApplicationHandle {
    pub(super) window_handles: HashMap<WindowIdentifier, WindowHandle>,
    pub(super) timers: HashMap<TimerToken, Timer>,
    pub(crate) event_listener: Option<Box<AppEventCallback>>,
    pub(super) gpu_resources: Option<GpuResources>,
    pub(crate) config: AppConfig,
}

/// Functionality that is not specific to a windowing system and can be implemented once for both, rather
/// than duplicate code.
///
/// The crate-facing API is implemented through `AppHandlerImpl` in `app_handle_baseview` or `app_handle_winit` depending
/// which windowing system is active.
impl AppHandlerCommon for ApplicationHandle {
    fn window_handle_for_window_id(
        &mut self,
        window_id: &WindowIdentifier,
    ) -> Option<&mut WindowHandle> {
        self.window_handles.get_mut(window_id)
    }

    fn handle_profile(
        &mut self,
        window_id: WindowIdentifier,
        end_profile: Option<WriteSignal<Option<Rc<Profile>>>>,
    ) {
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

    fn handle_theme_change(&mut self, theme: WindowSystemTheme) {
        self.config.global_theme_override = Some(theme);
        for window_handle in self.window_handles.values_mut() {
            window_handle.window_state.light_dark_theme = theme;
            window_handle.set_theme(Some(theme), false);
        }
    }

    fn handle_menu_action(&mut self, action_id: MenuId) {
        for (_, handle) in self.window_handles.iter_mut() {
            if handle.window_state.context_menu.contains_key(&action_id)
                || handle.window_menu_actions.contains_key(&action_id)
            {
                handle.menu_action(&action_id);
                break;
            }
        }
    }

    fn capture_window(&mut self, window_id: WindowIdentifier) -> Option<Capture> {
        self.window_handles
            .get_mut(&window_id)
            .map(|handle| handle.capture(self.gpu_resources.clone()))
    }

    fn publish_event(&mut self, event: AppEvent) {
        if let Some(action) = self.event_listener.as_ref() {
            action(event);
        }
    }

    fn take_expired_timers(&mut self) -> Vec<TimerToken> {
        let now = Instant::now();
        self.timers
            .iter()
            .filter_map(|(token, timer)| {
                if timer.deadline <= now {
                    Some(*token)
                } else {
                    None
                }
            })
            .collect()
    }

    fn remove_timer_instance(&mut self, token: &TimerToken) -> Option<Timer> {
        self.timers.remove(token)
    }
}
