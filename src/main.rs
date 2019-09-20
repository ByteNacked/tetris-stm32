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

const INPUT_ANALOG : u8 = 0;
const INPUT_FLOAT : u8 = 1;
const INPUT_PULL_UP_DOWN : u8 = 2;
fn port_init() {
    use stm32f1xx_hal::pac::{
        RCC,
        GPIOA,
        GPIOB,
        GPIOC,
        GPIOD,
        GPIOE,
        GPIOF,
        GPIOG
    };

    let rcc = unsafe { &*RCC::ptr() };
    rcc.apb2enr.modify(|_, w| {
        w.iopaen().enabled()
         .iopben().enabled()
         //.iopcen().enabled()
         .iopden().enabled()
         .iopeen().enabled()
         //.iopfen().enabled()
         //.iopgen().enabled()
    });

    let gpioa = unsafe { &*GPIOA::ptr() };
    gpioa.odr.modify(|_, w| {
        w.odr0().high()
         .odr1().high()
         .odr2().low()
         .odr3().high()
         .odr4().high()
         .odr5().low()
         .odr6().low()
         .odr7().low()
         .odr8().high()
         .odr9().high()
         .odr10().high()
         .odr11().low()
         .odr12().low()
    });

    gpioa.crl.modify(|_, w| {
        w.mode0().input()
         .cnf0().bits(INPUT_FLOAT)
         .mode1().output2()
         .cnf1().push_pull()
         .mode2().output2()
         .cnf2().push_pull()
         .mode3().input()
         .cnf3().bits(INPUT_FLOAT)
         .mode4().output2()
         .cnf4().push_pull()
         .mode5().output2()
         .cnf5().push_pull()
         .mode6().input()
         .cnf6().bits(INPUT_FLOAT)
         .mode7().output2()
         .cnf7().push_pull()
    });

    gpioa.crh.modify(|_, w| {
        w.mode8().input()
         .cnf8().bits(INPUT_FLOAT)
         .mode9().output2()
         .cnf9().push_pull()
         .mode10().input()
         .cnf10().bits(INPUT_PULL_UP_DOWN)
         .mode11().input()
         .cnf11().bits(INPUT_PULL_UP_DOWN)
         .mode12().input()
         .cnf12().bits(INPUT_PULL_UP_DOWN)
    });

   //TODO: registers 
}

fn fsmc_init() {
    use stm32f1xx_hal::pac::{
        RCC,
        FSMC,
        GPIOB,
        GPIOD,
        GPIOE,
    };
    
    let rcc = unsafe { &*RCC::ptr() };

    rcc.ahbenr.modify(|_, w| w.fsmcen().enabled());
    let fsmc = unsafe { &*FSMC::ptr() };
    fsmc.bcr1.modify(|_, w| {
        w.mbken().enabled()
         .muxen().enabled()
         .mtyp().flash()
         .mwid().bits16()
         .wren().enabled()
         .extmod().enabled()
    });
    fsmc.btr1.modify(|_, w| unsafe {
        w.addset().bits(12)
         .datast().bits(15)
         .addhld().bits(12)
         .accmod().a()
    });

   //TODO: comments
    let gpiob = unsafe { &*GPIOB::ptr() };
    let gpiod = unsafe { &*GPIOD::ptr() };
    let gpioe = unsafe { &*GPIOE::ptr() };
    gpiob.crl.modify(|_, w| {
        w.mode7().output2()
         .cnf7().alt_push_pull()
    });
    gpiod.crh.modify(|_, w| {
        w.mode14().output()
         .cnf14().alt_push_pull()
         .mode15().output()
         .cnf15().alt_push_pull()
    });
    gpiod.crl.modify(|_, w| {
        w.mode0().output()
         .cnf0().alt_push_pull()
         .mode1().output()
         .cnf1().alt_push_pull()
    });
    gpioe.crl.modify(|_, w| {
        w.mode7().output()
         .cnf7().alt_push_pull()
    });
    gpioe.crh.modify(|_, w| {
        w.mode8().output()
         .cnf8().alt_push_pull()
         .mode9().output()
         .cnf9().alt_push_pull()
         .mode10().output()
         .cnf10().alt_push_pull()
         .mode11().output()
         .cnf11().alt_push_pull()
         .mode12().output()
         .cnf12().alt_push_pull()
         .mode13().output()
         .cnf13().alt_push_pull()
         .mode14().output()
         .cnf14().alt_push_pull()
         .mode15().output()
         .cnf15().alt_push_pull()
    });
    gpiod.crh.modify(|_, w| {
        w.mode8().output()
         .cnf8().alt_push_pull()
         .mode9().output()
         .cnf9().alt_push_pull()
         .mode10().output()
         .cnf10().alt_push_pull()
         .mode11().output()
         .cnf11().alt_push_pull()
         .mode12().output()
         .cnf12().alt_push_pull()
         .mode13().output()
         .cnf13().alt_push_pull()
    });
    gpioe.crl.modify(|_, w| {
        w.mode3().output()
         .cnf3().alt_push_pull()
         .mode4().output()
         .cnf4().alt_push_pull()
    });
    gpiod.crl.modify(|_, w| {
        w.mode4().output2()
         .cnf4().alt_push_pull()
         .mode5().output2()
         .cnf5().alt_push_pull()
         .mode7().output2()
         .cnf7().alt_push_pull()
    });
}

#[entry]
fn main() -> ! {
    port_init();
    fsmc_init();

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