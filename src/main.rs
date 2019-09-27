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
    stm32::Interrupt,
    rtc::Rtc,
};
use port::*;
use pause::pause;
use lcd::{Lcd, LCD_WIDTH, LCD_HEIGHT, FULL_SCREEN_RECT, Rect, color::*};
use jlink_rtt::rtt_print;

//#[panic_handler]
//#[inline(never)]
//fn panic(_info: &PanicInfo) -> ! {
//    #[cfg(debug_assertions)]
//    cortex_m::asm::bkpt();
//    loop {}
//}

static mut TIMER_PAUSE: Option<Timer<pac::TIM1>> = None;
const RUST_LOGO : &'static[u8] = include_bytes!("../pic/rust-logo-white_t.bmp");
const RUST_EVA : &'static[u8] = include_bytes!("../pic/rust_eva_logo-t.bmp");
const RUST_EMB_240X289 : &'static[u8] = include_bytes!("../pic/rust_emb_240x289_t.bmp");
const RUST_BROWN : &'static[u8] = include_bytes!("../pic/rust_rust_200x200_t.bmp");

unsafe fn from_u8_slice(slice : &[u8]) -> &[u16] {
    use core::slice::from_raw_parts;
    use core::mem::{size_of, transmute};

    let ptr : * const u16 = unsafe { transmute(slice.as_ptr()) };
    let len : usize = slice.len() / (size_of::<u16>() / size_of::<u8>());

    unsafe { from_raw_parts(ptr, len)}
}


#[entry]
fn main() -> ! {
    port_init();
    fsmc_init();

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let mut nvic = cp.NVIC;
    //nvic.enable(Interrupt::USB_LP_CAN_RX0);

    let clocks = rcc
        .cfgr
        .freeze_72Mhz_nousb(&mut flash.acr);

    let mut gpiog = dp.GPIOG.split(&mut rcc.apb2);

    let mut led_red = gpiog.pg6.into_push_pull_output(&mut gpiog.crl);
    let mut led_green = gpiog.pg7.into_push_pull_output(&mut gpiog.crl);

    let mut pwr = dp.PWR;
    let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut rcc.apb1, &mut pwr);

    unsafe { RTC = Some(Rtc::rtc(dp.RTC, &mut backup_domain))};
    let mut rtc = unsafe { &mut RTC.as_mut().unwrap() };
    rtc.listen_seconds();
    nvic.enable(Interrupt::RTC);

    let mut systick = Timer::syst(cp.SYST, 1000.hz(), clocks);
    systick.listen(Event::Update);

    unsafe { TIMER_PAUSE = Some(Timer::tim1(dp.TIM1, 1.hz(), clocks, &mut rcc.apb2)) };

    unsafe { adc::ACCEL_ADC = Some(adc::Adc::new())};
    let adc = unsafe { adc::ACCEL_ADC.as_mut().unwrap() };
    adc.init();
    adc.start_conversion();

    let lcd = unsafe { &mut LCD };

    lcd.init();
    let rust_emb : &'static[u16] = unsafe { from_u8_slice(RUST_EMB_240X289) };
    lcd.fill_rect_with_bitmap(Rect{x : 0, y : 0, w : 289, h : 240}, rust_emb);
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());
    let rust_eva16 : &'static[u16] = unsafe { from_u8_slice(RUST_EVA) };
    lcd.fill_rect_with_bitmap(FULL_SCREEN_RECT, rust_eva16);
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());
    lcd.clear();
    let rust_logo16 : &'static[u16] = unsafe { from_u8_slice(RUST_LOGO) };
    lcd.fill_rect_with_bitmap(Rect { x : 75, y : 20, w : 200, h : 200}, rust_logo16);
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());
    pause(1000.ms());

    let mut green = unsafe { &mut GREEN };
    let mut red = unsafe { &mut RED };
    let mut blue = unsafe { &mut BLUE };
    lcd.fill_rect_with_color(*blue, 0b001111u16);
    lcd.fill_rect_with_color(*green, RGB(30, 220, 50));
    lcd.fill_rect_with_color(*red, Color::Red);

    unsafe { TIM2 = Some(Timer::tim2(dp.TIM2, 15.hz(), clocks, &mut rcc.apb1)) };
    let mut tim2 = unsafe { TIM2.as_mut().unwrap() };
    tim2.listen(Event::Update);
    unsafe { nvic.set_priority(Interrupt::TIM2, 3 << 4) };
    nvic.enable(Interrupt::TIM2);

    loop {
        pause(500.ms());
        let _ = led_green.set_high();
        let _ = led_red.set_low();
        pause(500.ms());
        let _ = led_green.set_low();
        let _ = led_red.set_high();
        //pause(30.ms());
    }
}

static mut BLUE : Rect = Rect{x : LCD_WIDTH - 50, y : LCD_HEIGHT - 70, w : 50, h : 70};
static mut GREEN : Rect = Rect{x : 100, y : 100, w : 25, h : 25};
static mut RED : Rect = Rect{x : 50, y : 50, w : 25, h : 25};
static mut LCD : Lcd = Lcd::new();
static mut RTC : Option<Rtc> = None;
static mut TIM2 : Option<Timer<pac::TIM2>> = None;

fn game_iter(tick : u32) {
    let mut green = unsafe { &mut GREEN };
    let mut red = unsafe { &mut RED };
    let mut blue = unsafe { &mut BLUE };

    let mut lcd = unsafe { &mut LCD };

    let prev_red = *red;
    let prev_green = *green;

    unsafe { adc::ACCEL_ADC.as_mut().unwrap().start_conversion() };
    let (_, y, x) = unsafe { adc::ACCEL_ADC.as_mut().unwrap().get_axes() };
    let dy = if y < 50 && y > -50 { 0 } else { y };
    let dx = if x < 50 && x > -50 { 0 } else { x };

    let yy : isize = red.y as isize + (dy / 8);
    red.y = if yy + red.h as isize > LCD_HEIGHT as isize {
        LCD_HEIGHT - red.h
    } 
    else if yy < 0 { 0 }
    else { yy as usize };

    let xx : isize = green.x as isize + 20 * dx.signum();
    green.x = if xx + green.w as isize > LCD_WIDTH as isize {
        LCD_WIDTH - green.w
    } 
    else if xx < 0 { 0 }
    else { xx as usize };

    cortex_m::interrupt::free(|_| {
        lcd.fill_rect_with_color(prev_red, Color::Black);
        lcd.fill_rect_with_color(prev_green, Color::Black);
        lcd.fill_rect_with_color(*red, Color::Red);
        lcd.fill_rect_with_color(*green, Color::Green);
    });

    //if tick % 100 == 0 {
    //    let _ = rtt_print!("{:?}", unsafe { adc::ACCEL_ADC.as_ref().unwrap() } );
    //    let _ = rtt_print!("Red : {:?}", red);
    //    let _ = rtt_print!("Grn : {:?}", green);
    //    let _ = rtt_print!("y : {:?}, x : {:?}", &y, &x);
    //}
}


static mut DBG_TICK : u32 = 0;
//INTERRUPTS
#[interrupt]
fn TIM2() {
    unsafe { TIM2.as_mut().unwrap().clear_update_interrupt_flag() };

    game_iter(unsafe { DBG_TICK });
    unsafe { DBG_TICK += 1; }
}

#[interrupt]
fn RTC() {
    let mut rtc = unsafe { &mut RTC.as_mut().unwrap() };
    rtc.clear_second_flag();

    let res = unsafe { DBG_TICK };
    unsafe { DBG_TICK = 0; }

    //let _ = rtt_print!("TIM2 hz: {}", res);
}

#[exception]
fn SysTick() {
    static mut DBG_SYSTICK : u32 = 0;
    *DBG_SYSTICK += 1;
}