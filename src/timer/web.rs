//! Web based timer

use core::{task, time};
use core::pin::Pin;
use core::future::Future;

use crate::state::TimerState;
use crate::alloc::boxed::Box;

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn setTimeout(closure: &wasm_bindgen::closure::Closure<dyn FnMut()>, time: u32) -> i32;
    fn clearTimeout(id: i32);
}

struct TimerHandle(i32);

impl TimerHandle {
    #[inline]
    fn clear(&mut self) {
        clearTimeout(self.0)
    }
}

impl Drop for TimerHandle {
    fn drop(&mut self) {
        self.clear();
    }
}

fn time_create(timeout: time::Duration, state: *const TimerState) -> TimerHandle {
    let timeout = timeout.as_millis() as u32;

    let cb = wasm_bindgen::closure::Closure::once(move || unsafe {
        (*state).wake();
    });
    let handle = setTimeout(&cb, timeout);

    TimerHandle(handle)
}

enum State {
    Init(time::Duration),
    Running(TimerHandle, *const TimerState),
}

unsafe impl Send for State {}
unsafe impl Sync for State {}

///Web timer wrapper
pub struct WebTimer {
    state: State,
}

impl WebTimer {
    #[inline]
    ///Creates new instance
    pub const fn new(time: time::Duration) -> Self {
        Self {
            state: State::Init(time),
        }
    }
}

impl super::Timer for WebTimer {
    #[inline(always)]
    fn new(timeout: time::Duration) -> Self {
        assert_time!(timeout);
        Self::new(timeout)
    }

    #[inline]
    fn is_ticking(&self) -> bool {
        match &self.state {
            State::Init(_) => false,
            State::Running(_, ref state) => unsafe {
                !(**state).is_done()
            },
        }
    }

    #[inline]
    fn is_expired(&self) -> bool {
        match &self.state {
            State::Init(_) => false,
            State::Running(_, ref state) => unsafe {
                (**state).is_done()
            },
        }
    }

    fn restart(&mut self, new_value: time::Duration) {
        assert_time!(new_value);

        match &mut self.state {
            State::Init(ref mut timeout) => {
                *timeout = new_value;
            },
            State::Running(fd, ref state) => {
                unsafe { (**state).reset() };
                *fd = time_create(new_value, *state);
            }
        }
    }

    fn cancel(&mut self) {
        match self.state {
            State::Init(_) => (),
            State::Running(ref mut fd, _) => {
                fd.clear()
            }
        }
    }
}

impl super::SyncTimer for WebTimer {
    fn init<R, F: Fn(&TimerState) -> R>(&mut self, init: F) -> R {
        if let State::Init(timeout) = self.state {
            let state = TimerState::new();
            init(&state);

            let state = Box::into_raw(Box::new(state));
            let fd = time_create(timeout, state);

            self.state = State::Running(fd, state)
        }

        match &self.state {
            State::Running(_, ref state) => init(unsafe { &**state }),
            State::Init(_) => unreach!(),
        }
    }
}

impl Future for WebTimer {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context) -> task::Poll<Self::Output> {
        crate::timer::poll_sync(self.get_mut(), ctx)
    }
}

impl Drop for WebTimer {
    fn drop(&mut self) {
        match self.state {
            State::Running(ref mut fd, state) => {
                fd.clear();
                unsafe { Box::from_raw(state as *mut TimerState) };
            },
            _ => (),
        }
    }
}