use std::{cell::RefCell, rc::Rc};
use adapters::WindowSystemTheme;
use floem_reactive::WriteSignal;
use muda::MenuId;
use windowing::public_api::WindowIdentifier;
#[cfg(feature = "baseview")]
use crate::AnyView;
use crate::{action::{Timer, TimerToken}, inspector::Capture, profiler::Profile, window::WindowCreation, Application};

pub(crate) type AppEventCallback = dyn Fn(AppEvent);

thread_local! {
    static APP_UPDATE_EVENTS: RefCell<Vec<AppUpdateEvent>> = Default::default();
}

pub enum AppEvent {
    WillTerminate,
    Reopen { has_visible_windows: bool },
}

#[derive(Debug)]
pub(crate) enum UserEvent {
    AppUpdate,
    Idle,
    QuitApp,
    #[allow(dead_code)]
    Reopen {
        has_visible_windows: bool,
    },
    GpuResourcesUpdate {
        window_id: WindowIdentifier,
    },
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum AppUpdateEvent {
    NewWindow {
        window_creation: WindowCreation,
    },
    CloseWindow {
        window_id: WindowIdentifier,
    },
    CaptureWindow {
        window_id: WindowIdentifier,
        capture: WriteSignal<Option<Rc<Capture>>>,
    },
    ProfileWindow {
        window_id: WindowIdentifier,
        end_profile: Option<WriteSignal<Option<Rc<Profile>>>>,
    },
    RequestTimer {
        timer: Timer,
    },
    CancelTimer {
        timer: TimerToken,
    },
    MenuAction {
        action_id: MenuId,
    },
    ThemeChanged {
        theme: WindowSystemTheme,
    },
}

pub(crate) fn add_app_update_event(event: AppUpdateEvent) {
    APP_UPDATE_EVENTS.with(|events| {
        events.borrow_mut().push(event);
    });
    Application::send_proxy_event(UserEvent::AppUpdate);
}

pub(crate) fn retreive_app_update_events() -> Vec<AppUpdateEvent> {
    APP_UPDATE_EVENTS.with(|events| {
        let mut events = events.borrow_mut();
        std::mem::take(&mut *events)
    })
}
