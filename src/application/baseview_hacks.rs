use std::{ops::{Deref, DerefMut}, thread};
use baseview::{*, gl::GlContext};
use peniko::kurbo::Size;

/*
Okay, this is a monstrous hack just to get things working.  We are passed a &mut Window,
and it needs to get threaded through a bunch of calls.  So, this just gives us something
that will exist ephemerally, withing the scope of the closure of a windowing call and then
vanish.

We should find a better way to do this.
*/

pub(crate) const NO_WINDOW : BaseviewPseudoEventLoop = BaseviewPseudoEventLoop { window : 0 };

// hope we don't have to use this. Welp, we do if we don't want to change the API.
thread_local! {
    static CURRENT_WINDOW : std::cell::RefCell<BaseviewPseudoEventLoop> = std::cell::RefCell::new(NO_WINDOW);
}

pub(crate) fn setting_current_window<F: FnOnce() -> T, T>(window : BaseviewPseudoEventLoop, f : F) -> T {
    CURRENT_WINDOW.with(|cell| {
        cell.replace(window);
        let result = f();
        cell.replace(NO_WINDOW);
        result
    })
}

pub(crate) fn current_window() -> BaseviewPseudoEventLoop {
    CURRENT_WINDOW.with(|c| {
        c.borrow().to_owned()
    })
}

/// This is an ephemeral wrapper for the `Window` that gets passed into our listeners - the only time it
/// can be accessed is within the closure of a listener method.  Floem has lots of indirection that makes
/// it difficult to pass around - we are using at as the "event loop" parameter, since that is similar
/// to what you get from winit - the `ActiveEventLoop` gets passed by reference, but we don't and can't
/// hold a reference to it outside the closure of an event - so the *access pattern* is the same.
/// This type does pure evil pointer magic that is very likely thoroughly unsound, but it will do to
/// get something at least running.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct BaseviewPseudoEventLoop {
    // This is hideous
    window : usize,
}

impl BaseviewPseudoEventLoop {

    pub fn is_none(&self) -> bool {
        self.window == 0
    }

    pub fn close(&self) {
        self.with_mut(|w| w.close());
    }

    pub fn resize(&self, size: Size) {
        self.with_mut(|w| w.resize(baseview::Size {
            width: size.width,
            height: size.height,
        })).unwrap_or(Default::default())
    }

    pub fn has_focus(&self) -> bool {
        self.with_mut(|w| w.has_focus()).unwrap_or_default()
    }

    pub fn focus(&mut self) {
        self.with_mut(Window::focus);
    }

    pub fn gl_context(&self) -> Option<&baseview::gl::GlContext> {
        let r = unsafe { &*(self.window as *const Window<'_>) };
        r.gl_context()
    }

    pub(crate) fn with_mut<'r: 'l, 'l, T>(&'r self, f : impl FnOnce(&mut Window<'l>) -> T) -> Option<T> {
        if self.window != 0 {
            let r = unsafe { &mut * (self.window as *mut Window<'l>) };
            Some(f(r))
        } else {
            None
        }
    }

    pub(crate) fn with_ref<'r: 'l, 'l, T>(&'r self, f : impl FnOnce(&Window<'l>) -> T) -> Option<T> {
        if self.window != 0 {
            let r = unsafe { &*(self.window as *const Window<'l>) };
            Some(f(r))
        } else {
            None
        }
    }
}

impl<'l> From<&mut Window<'l>> for BaseviewPseudoEventLoop {
    fn from(value: &mut Window<'l>) -> Self {
        let pt : *mut Window<'l> = value;
        Self {
            window : pt as usize,
        }
    }
}
