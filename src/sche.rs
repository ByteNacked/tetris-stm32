
use core::ops::{Generator, GeneratorState};
use core::pin::Pin;

use cortex_m::asm;
use cortex_m::asm::{delay, wfi, bkpt};
use cortex_m_rt::{entry, exception};
use super::rtt_print;

const fn up_to(limit: u32) -> impl Generator<Yield = u32, Return = u32> + core::marker::Unpin + core::marker::Sized {
    move || {
        let mut i = 0;
        loop {
            i += 1;
            yield i;
        }
        return limit;
    }
}



fn test() {
    unsafe {
        asm!("NOP" : : : : "volatile");
        asm!("NOP");
        asm!("NOP");
    }
}

static mut MY_GEN : impl Generator<Yield = u32, Return = u32> + core::marker::Unpin = up_to(2);


pub fn schedule() {

    rtt_print!("Gen size {}", core::mem::size_of_val(unsafe{&MY_GEN}));
    
    match Pin::new(unsafe {&mut MY_GEN}).resume() {
        GeneratorState::Yielded(num) => { rtt_print!("Step : {}", num); }
        GeneratorState::Complete(_) => { rtt_print!("Finish step!"); }
        _ => panic!("unexpected value from resume"),
    }
}


#[exception]
fn SVCall() {
    //asm!("
    //    tst lr, #4
    //    ite eq
    //    mrseq r0, msp
    //    mrsne r0, psp
    //    ldr r1, [r0, #24]
    //    ldrb r1, [r1, #-2]
    //    ldr pc, [r2, r1, lsl #2]
    //"   :
    //    : "{r2}"(T::first())
    //    : "r0", "r1", "cc"
    //    : "volatile"
    //);



}

#[exception]
fn PendSV() {
    test();
}