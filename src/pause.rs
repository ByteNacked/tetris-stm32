
use super::TIMER_PAUSE;
use stm32f1xx_hal::{
    prelude::*,
    time::MilliSeconds,
};
use nb::block;

pub fn pause(mili : MilliSeconds) {
    unsafe {
        if let Some(t) = TIMER_PAUSE.as_mut() {
            t.start(mili);
            let _ = block!(t.wait());
        }
    }
}