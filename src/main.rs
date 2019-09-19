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

#[derive(Debug)]
struct TestA {
    a : u32,
    b : i8,
    c : &'static str,
    d : [u8; 3],
    e : InnerB<'static>
}

#[derive(Debug)]
enum TestVar {
    Good,
    Bad,
    Ugly,
}

#[derive(Debug)]
struct InnerB<'a> {
    a : &'a str,
    b : TestVar,
}

const TEST_STR : &'static str = "yellow tree";
const TEST_A : TestA = TestA{
    a : 246u32,
    b : -8,
    c : TEST_STR,
    d : [3u8, 2u8, 1u8],
    e : InnerB {
        a : "boom",
        b : TestVar::Ugly,
    },
};

fn port_init(dp : pac::Peripherals) {
    let rcc = dp.RCC;
    //rcc.ahb2enr.write


    //let mut gpiog = dp.GPIOG.split(rcc.apb2);
    //let mut led_red = gpiog.pg6.into_push_pull_output(&mut gpiog.crl);
    //let mut led_green = gpiog.pg7.into_push_pull_output(&mut gpiog.crl);

}

fn fsmc_init(dp : &pac::Peripherals) {

}

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
        let _ = writeln!(&mut output, "Hello {:?}", &TEST_A);
    }

    let mut timer = Timer::syst(cp.SYST, 10.hz(), clocks);

    loop {
        block!(timer.wait()).unwrap();
        led_green.set_high();
        led_red.set_low();
        block!(timer.wait()).unwrap();
        led_green.set_low();
        led_red.set_high();
    }
}