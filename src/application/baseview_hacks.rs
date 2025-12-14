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

// hope we don't have to use this
thread_local! {
    static CURRENT_WINDOW : std::cell::RefCell<BaseviewPseudoEventLoop> = std::cell::RefCell::new(NO_WINDOW);
}

pub(crate) fn setting_current_window<F: FnOnce()>(window : BaseviewPseudoEventLoop, f : F) {
    CURRENT_WINDOW.with(|cell| {
        cell.replace(window);
        f();
        cell.replace(NO_WINDOW);
    })
}

pub(crate) fn current_window() -> BaseviewPseudoEventLoop {
    CURRENT_WINDOW.with(|c| {
        c.borrow().to_owned()
    })
}

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct BaseviewPseudoEventLoop {
    // This is hideous
    window : usize,
}

impl BaseviewPseudoEventLoop {
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
        /*
        let op : Option<Option<&baseview::gl::GlContext>> = self.with_ref(|w| {
            w.gl_context()
        });
        if let Some(op) = op {
            return op
        }
        None
         */
    }

    fn with_mut<'r: 'l, 'l, T>(&'r self, f : impl FnOnce(&mut Window<'l>) -> T) -> Option<T> {
        if self.window != 0 {
            let r = unsafe { &mut * (self.window as *mut Window<'l>) };
            Some(f(r))
        } else {
            None
        }
    }

    fn with_ref<'r: 'l, 'l, T>(&'r self, f : impl FnOnce(&Window<'l>) -> T) -> Option<T> {
        if self.window != 0 {
            let r = unsafe { &*(self.window as *const Window<'l>) };
            Some(f(r))
        } else {
            None
        }
    }
}
/*
impl<'l> Deref for BaseviewPseudoEventLoop {
    type Target = Window<'l>;

    fn deref(&self) -> &Self::Target {
        &(self.window as *const Window<'l>)
    }
}

impl<'l> DerefMut for BaseviewPseudoEventLoop {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (self.window as *mut Window<'l>);
    }
}
 */

// impl<'l> From<&'l mut Window<'l>> for BaseviewPseudoEventLoop {
//     fn from(value: &'l mut Window<'l>) -> Self {
//         let pt : *mut Window<'l> = value;
//         Self {
//             window : pt as usize,
//         }
//     }
// }
impl<'l> From<&mut Window<'l>> for BaseviewPseudoEventLoop {
    fn from(value: &mut Window<'l>) -> Self {
        let pt : *mut Window<'l> = value;
        Self {
            window : pt as usize,
        }
    }
}
