use baseview::WindowHandle;

/*
Hmm, baseview::Window has a lifetime.

Maybe use slotmap to manage the instance?  Or cast to 'static and cheat - in theory,
at least the main window must outlive any rust code rendering into it.  A dynamically
created popup window is more questionable.  And more than one instance will not have the
same lifetime, so there may be no avoiding some transmuting.

A real solution would be to eliminate the lifetime in baseview; it appears it is only
needed for Windows, and the thing that actually uses the lifetime a few layers down is
the os-allocated window handle.  Lifetimes have limited meaning for things like that,
and it might as well be a pointer because in reality, Rust code knows nothing about when
the host application or OS might choose to deallocate it - its allocation is beyond Rust's
control in the first place.

*/

type WindowIdInner = u16;

/// A transparent wrapper over the library handling window management's window
/// identity abstraction.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct WindowIdentifier {
    id: WindowIdInner,
}
