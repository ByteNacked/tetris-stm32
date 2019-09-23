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
use cortex_m::asm::{delay, wfi, bkpt};
use cortex_m_rt::{entry, exception};
use nb::block;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{
    prelude::*,
    pac,
    timer::{ Timer, Event},
    stm32,
    stm32::interrupt,
    stm32::Interrupt
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


static mut TIMER_PAUSE: Option<Timer<pac::TIM1>> = None;



#[entry]
fn main() -> ! {
    port_init();
    fsmc_init();

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    //let mut nvic = cp.NVIC;
    //nvic.enable(Interrupt::USB_LP_CAN_RX0);

    let clocks = rcc
        .cfgr
        .freeze_72Mhz_nousb(&mut flash.acr);

    let mut gpiog = dp.GPIOG.split(&mut rcc.apb2);

    let mut _led_red = gpiog.pg6.into_push_pull_output(&mut gpiog.crl);
    let mut _led_green = gpiog.pg7.into_push_pull_output(&mut gpiog.crl);

    //{//TODO: move to macro
    //    use core::fmt::Write;
    //    let mut output = jlink_rtt::Output::new();
    //    let _ = writeln!(&mut output, "Hello {:?}", &TEST_A);
    //}

    let mut systick = Timer::syst(cp.SYST, 1000.hz(), clocks);
    systick.listen(Event::Update);

    unsafe {
        if let None = TIMER_PAUSE {
            TIMER_PAUSE = Some(Timer::tim1(dp.TIM1, 1.hz(), clocks, &mut rcc.apb2));
        }
    }

    unsafe {
        if let None = adc::ACCEL_ADC {
            adc::ACCEL_ADC = Some(adc::Adc::new());
            adc::ACCEL_ADC.as_mut().unwrap().init();
            adc::ACCEL_ADC.as_mut().unwrap().start_conversion();
        }
    }


    let mut lcd = Lcd::new();
    lcd.init();
    lcd.fill_rect_with_color(Rect{x : LCD_WIDTH - 50, y : LCD_HEIGHT - 70, w : 50, h : 70}, 0b001111u16);
    lcd.fill_rect_with_color(Rect{x : 100, y : 100, w : 50, h : 70}, RGB(30, 220, 50));
    let mut red = Rect{x : 50, y : 50, w : 50, h : 70};
    lcd.fill_rect_with_color(red, Color::Red);
    loop {
        //pause(500.ms());
        //let _ = led_green.set_high();
        //let _ = led_red.set_low();
        //pause(500.ms());
        //let _ = led_green.set_low();
        //let _ = led_red.set_high();

        pause(1000.ms());
        lcd.fill_rect_with_color(red, Color::Black);
        unsafe { adc::ACCEL_ADC.as_mut().unwrap().start_conversion() };
        let (_, y, _) = unsafe { adc::ACCEL_ADC.as_mut().unwrap().get_axes() };
        let y = if y < 50 && y > -50 { 0 } else { y };

        let yy = red.y as isize + (y / 4) * y.signum();
        if  yy < LCD_HEIGHT as isize && yy >= 0 {
            red.y = yy as usize;
        }

        lcd.fill_rect_with_color(red, Color::Red);
        use core::fmt::Write;
        let mut output = jlink_rtt::Output::new();
        let _ = writeln!(&mut output, "{:?}", unsafe { adc::ACCEL_ADC.as_ref().unwrap() } );
        let _ = writeln!(&mut output, "{:?}", &red);
        let _ = writeln!(&mut output, "{:?}", &y);

    }
}

//INTERRUPTS
#[interrupt]
fn TIM2() {
    static mut DBG_SYSTICK : u32 = 0;
    *DBG_SYSTICK += 1;
}

#[exception]
fn SysTick() {
    static mut DBG_SYSTICK : u32 = 0;
    *DBG_SYSTICK += 1;
}