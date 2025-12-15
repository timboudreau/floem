//! Baseview has no equivalent of winit's `ActiveEventLoop`'s pause and resume
//! behavior.  This module provides a simple equivalent of the control flow portion of that.
//! `WindowHandler::on_frame()` uses it to determine whether to call the rest of the plumbing
//! of the application based on it.
use std::{
    ops::Deref, sync::{
        atomic::{
            AtomicU64, AtomicU8, Ordering::{Acquire, Release}
        },
        LazyLock,
    }, time::{Duration, Instant}
};

static EPOCH: LazyLock<Instant> = LazyLock::new(|| Instant::now());
static CONTROL_FLOW: AtomicU64 = AtomicU64::new(1);
static IS_CHANGE : AtomicU8 = AtomicU8::new(0);

/// An equivalent to winit's control flow - global, and done in the simplest
/// way possible, storing all of its state in a single atomic `u64`.
///
/// The top two bits hold the state - poll, wait or wait until; the lower
/// 62 bits record an offset in milliseconds from the time the field `EPOCH`
/// was first accessed.  Unless the application is running for around 292 billion
/// years, these bits will not collide, and millisecond-resolution is more than
/// adequate for GUI apps, given human perceptual limitations.
///
/// There is only one application per process, so there is no need to attach
/// it to a window or context.  Call `set` to set it, call `get` to read the current
/// value.
/// Access is effectively atomic, lockless after the first call per the contract of
/// `LazyLock`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ControlFlow {
    Poll,
    Wait,
    WaitUntil(Instant),
}

impl ControlFlow {
    /// Get the current global control flow value
    pub fn get() -> Self {
        CONTROL_FLOW.load(Acquire).into()
    }

    /// Get the deadline for execution resumption of this `ControlFlow`, returning an
    /// arbitrary past instant for `Poll`, an arbitrary future instant for `Wait`,
    /// and the explicit deadline of a `WaitUntil` instance (which is stored at millisecond
    /// resolution, and may vary by up to 1ms from the original value, if this instance was
    /// retrieved via a call to `ControlFlow::get`).
    pub fn deadline(&self) -> Instant {
        match self {
            Self::Poll => Instant::now() - Duration::from_millis(1),
            Self::Wait => Instant::now() + Duration::from_hours(876000), // 100 years should be enough, right?
            Self::WaitUntil(deadline) => *deadline,
        }
    }

    /// Set this instance as the current global control flow value
    pub fn set(&self) {
        CONTROL_FLOW.store((*self).into(), Release);
    }

    /// Determine if execution is paused
    pub fn is_paused(&self) -> bool {
        match self {
            Self::Wait => true,
            Self::Poll => false,
            Self::WaitUntil(target) => Instant::now() < *target,
        }
    }

    /// Internal call which reports the paused state of this instance *and stores a flag
    /// if a preceding call on any instance reported itself paused and this one does not.
    /// This should be called by a single site which checks the current state to decide
    /// whether to proceed with execution or not.
    pub(super) fn check_paused(&self) -> bool {
        unpause_check(self.is_paused())
    }

    /// Determine if this is a `WaitUntil` instance with an explicit deadline.
    pub fn has_deadline(&self) -> bool {
        matches![self, Self::WaitUntil(_)]
    }

    /// Determine if, according to the contract of `check_paused`, execution just resumed
    /// from a paused state.
    pub fn is_resumed() -> bool {
        was_just_unpaused()
    }
}

fn unpause_check(paused : bool) -> bool {
    let uvalue : u8 = if paused {
        1
    } else {
        0
    };
    // Store 0b01 if paused, 0b00 if not paused, unless less the previous value was
    // paused and the new value is not, in which case 0b10 is stored - bit 2 indicates
    // a state change, which will persist only until the next call to this function.
    IS_CHANGE.fetch_update(Acquire, Acquire, |old_val| {
        if old_val == uvalue {
            None
        } else if !paused {
            if old_val & 2 != 0 {
                Some(uvalue)
            } else {
                Some(uvalue | 2)
            }
        } else {
            Some(uvalue)
        }
    });
    paused
}

fn was_just_unpaused() -> bool {
    IS_CHANGE.load(Acquire) & 2 != 0
}

fn dawn_of_time() -> Instant {
    EPOCH.deref().to_owned()
}

impl From<ControlFlow> for u64 {
    fn from(value: ControlFlow) -> Self {
        match value {
            ControlFlow::Poll => 1_u64 << 62,
            ControlFlow::Wait => 2_u64 << 62,
            ControlFlow::WaitUntil(instant) => {
                let state = 3_u64 << 62;
                let when = instant
                    .saturating_duration_since(dawn_of_time())
                    .as_millis() as u64
                    & 0x3fffffffffffffff;
                state | when
            }
        }
    }
}

impl From<u64> for ControlFlow {
    fn from(value: u64) -> Self {
        let state_bits = (value & 0xc000000000000000) >> 62;
        match state_bits {
            2 => Self::Wait,
            3 => {
                let time_bits = value & 0x3fffffffffffffff;
                let dur = std::time::Duration::from_millis(time_bits);
                let deadline = dawn_of_time() + dur;
                Self::WaitUntil(deadline)
            }
            _ => Self::Poll,
        }
    }
}

fn now_millis() -> u64 {
    let dawn_of_time = dawn_of_time();
    let il = Instant::now();
    let dur = il.saturating_duration_since(dawn_of_time);
    dur.as_millis() as u64 & 0x00
}

#[cfg(test)]
mod control_flow_tests {
    use serial_test::serial;
    use std::{time::Duration, u64};
    use super::*;

    #[test]
    fn test_simple_bits() {
        let a : u64 = ControlFlow::Poll.into();
        assert_eq!(1 << 62, a);

        let b: u64 = ControlFlow::Wait.into();
        assert_eq!(2 << 62, b);
    }

    #[test]
    fn test_deadline_poll() {
        let f = ControlFlow::Poll;
        assert!(f.deadline() < Instant::now());
    }

    #[test]
    fn test_deadline_wait() {
        let f = ControlFlow::Wait;
        assert!(f.deadline() > Instant::now());
    }

    #[test]
    fn test_deadline_wait_until() {
        // Get the epoch initialized
        let _ = dawn_of_time();
        let one_ms = Duration::from_millis(1);
        // Ensure that no matter how fast the machine is, we're at least one milliseconds
        // into the future from epoch.
        std::thread::sleep(one_ms);

        let i = Instant::now();
        let c = ControlFlow::WaitUntil(i);
        let one_ms = Duration::from_millis(1);

        let raw : u64 = c.into();
        assert!((raw & & 0x3fffffffffffffff) != 0, "Wait until should encode 62 bits of milliseconds-since-epoch but contains zeros");

        let deadline = c.deadline();
        let delta = if let Some(fwd) = deadline.checked_duration_since(i) {
            fwd
        } else if let Some(bwd) = i.checked_duration_since(deadline) {
            bwd
        } else {
            unreachable!("Should always be able to get a zero duration. checked_duration_since is broken.");
        };
        assert!(delta <= one_ms, "62 bit millisecond-based stored instant should be within 1ms of provided value");
    }

    #[test]
    #[serial]
    fn test_initial_state_is_poll() {
        // If we are run with the cargo test single-threaded argument, our state
        // can be unknown
        CONTROL_FLOW.store(0, std::sync::atomic::Ordering::SeqCst);
        IS_CHANGE.store(0, std::sync::atomic::Ordering::SeqCst);
        let flow = ControlFlow::get();
        assert!(matches![flow, ControlFlow::Poll], "Got {:?}", flow);
        assert!(!flow.is_paused());
    }

    #[test]
    #[serial]
    fn test_switch_to_paused() {
        let flow = ControlFlow::get();
        ControlFlow::Wait.set();
        let flow = ControlFlow::get();
        assert!(matches![flow, ControlFlow::Wait], "Got {:?}", flow);
        assert!(flow.is_paused());
    }

    #[test]
    #[serial]
    fn test_deadline() {
        assert!(!was_just_unpaused());
        let deadline = Instant::now() + Duration::from_millis(200);
        ControlFlow::WaitUntil(deadline).set();

        let mut paused_seen = false;
        let mut was_change = false;
        let mut ct = 0;
        loop {
            let now = Instant::now();
            let flow = ControlFlow::get();
            ct += 1;
            assert!(flow.has_deadline());
            let paused = flow.check_paused();
            if paused {
                paused_seen = true;
            }
            if !paused {
                was_change = ControlFlow::is_resumed();
                let paused_2 = flow.check_paused();
                assert!(!paused_2);
                assert!(!ControlFlow::is_resumed(), "was_just_unpaused should only return true once if flow.is_paused() has been called a second time");
                break;
            }
        }
        assert!(paused_seen, "Never saw a paused state");
    }
}
