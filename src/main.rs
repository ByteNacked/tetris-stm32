#![no_std]
#![no_main]

extern crate jlink_rtt;
extern crate panic_rtt;

mod lcd;
mod port;
mod pause;
mod adc;

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
use port::*;
use pause::pause;
use lcd::{Lcd, LCD_WIDTH, LCD_HEIGHT, Rect, color::*};

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

static mut TIMER_PAUSE: Option<Timer<pac::TIM1>> = None;

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

    let mut _systick = Timer::syst(cp.SYST, 1000.hz(), clocks);

    unsafe {
        if let None = TIMER_PAUSE {
            TIMER_PAUSE = Some(Timer::tim1(dp.TIM1, 1.hz(), clocks, &mut rcc.apb2));
        }
    }

    let mut lcd = Lcd::new();
    lcd.init();
    lcd.fill_rect_with_color(Rect{x : LCD_WIDTH - 50, y : LCD_HEIGHT - 70, w : 50, h : 70}, 0b001111u16);
    lcd.fill_rect_with_color(Rect{x : 50, y : 50, w : 50, h : 70}, Color::Red);
    lcd.fill_rect_with_color(Rect{x : 100, y : 100, w : 50, h : 70}, RGB(30, 220, 50));


    loop {
        pause(500.ms());
        let _ = led_green.set_high();
        let _ = led_red.set_low();
        pause(500.ms());
        let _ = led_green.set_low();
        let _ = led_red.set_high();
    }
}