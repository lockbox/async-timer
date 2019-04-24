//! Dummy timer which is used on platforms without own implementation

use crate::{TimerState, Timer};

use core::time;

///Dummy timer.
///
///Holds no particular implementation and panics when used.
pub struct DummyTimer {
}

impl Timer for DummyTimer {
    fn new() -> Self {
        unimplemented!()
    }

    fn reset(&mut self) {
        unimplemented!()
    }

    fn start_delay(&mut self, _: time::Duration) {
        unimplemented!()
    }

    fn start_interval(&mut self, _: time::Duration) {
        unimplemented!()
    }

    fn state(&self) -> &TimerState {
        unimplemented!()
    }
}
