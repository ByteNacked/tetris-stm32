#![no_std]
#![no_main]

extern crate jlink_rtt;
extern crate panic_rtt;

use core::panic::PanicInfo;
use cortex_m::asm;
use cortex_m_rt::entry;
use nb::block;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{
    prelude::*,
    pac,
    timer::Timer,
    stm32,
};

//#[panic_handler]
//#[inline(never)]
//fn panic(_info: &PanicInfo) -> ! {
//    #[cfg(debug_assertions)]
//    cortex_m::asm::bkpt();
//    loop {}
//}

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .freeze_72Mhz_nousb(&mut flash.acr);

    let mut gpiog = dp.GPIOG.split(&mut rcc.apb2);

    let mut led_red = gpiog.pg6.into_push_pull_output(&mut gpiog.crl);
    let mut led_green = gpiog.pg7.into_push_pull_output(&mut gpiog.crl);

    {//TODO: move to macro
        use core::fmt::Write;
        let mut output = jlink_rtt::Output::new();
        let _ = writeln!(&mut output, "Hello {}", 42);
    }

    let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);

    loop {
        block!(timer.wait()).unwrap();
        led_green.set_high();
        led_red.set_low();
        block!(timer.wait()).unwrap();
        led_green.set_low();
        led_red.set_high();
    }
}