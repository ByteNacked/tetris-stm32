#![no_std]
#![no_main]

#![feature(asm)]
#![feature(generators, generator_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(const_fn)]
#![feature(never_type)]

#![allow(unused_imports)]
#![allow(incomplete_features)]

extern crate jlink_rtt;
extern crate panic_rtt;

mod lcd;
mod port;
mod pause;
mod adc;
mod sche;
mod beeper;
mod embbox;
mod splash;

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
use embbox::EmbBox;

//#[panic_handler]
//#[inline(never)]
//fn panic(_info: &PanicInfo) -> ! {
//    #[cfg(debug_assertions)]
//    cortex_m::asm::bkpt();
//    loop {}
//}


use core::ops::{Generator, GeneratorState};
use core::pin::Pin;

struct Valve {
    pub reg : u32,
}

#[entry]
fn main() -> ! {
    rtt_print!("Test 1");

    let mut v = Valve { reg : 0, };

    let mut sb : EmbBox< dyn Generator<Yield = u32, Return = !> + core::marker::Unpin, [usize; 8]> = embbox!{
        || {
            
            loop {
                yield 0u32;
                v.reg = 2;
                yield 1;
                v.reg = 3;
                yield 2;
            }
        }
    };

    rtt_print!("Test 2");
    for _ in 0 .. 3 {
        match Pin::new(&mut *sb).resume() {
            GeneratorState::Yielded(num) => { rtt_print!("Step : {}", num); }
            GeneratorState::Complete(_) => { rtt_print!("Finish step!"); }
        }
    }

    rtt_print!("Test 3");

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
    let rtc = unsafe { &mut RTC.as_mut().unwrap() };
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

    splash::draw_sreens(lcd);

    //let green = unsafe { &mut GREEN };
    //let red = unsafe { &mut RED };
    //let blue = unsafe { &mut BLUE };
    //lcd.fill_rect_with_color(*blue, 0b001111u16);
    //lcd.fill_rect_with_color(*green, RGB(30, 220, 50));
    //lcd.fill_rect_with_color(*red, Color::Red);

    unsafe { TIM2 = Some(Timer::tim2(dp.TIM2, 15.ms(), clocks, &mut rcc.apb1)) };
    let tim2 = unsafe { TIM2.as_mut().unwrap() };
    tim2.listen(Event::Update);
    unsafe { nvic.set_priority(Interrupt::TIM2, 3 << 4) };

    create_tetris();
    nvic.enable(Interrupt::TIM2);

    loop {
        pause(500.ms());
        let _ = led_green.set_high();
        let _ = led_red.set_low();
        pause(500.ms());
        let _ = led_green.set_low();
        let _ = led_red.set_high();
    }
}

static mut BLUE : Rect = Rect{x : LCD_WIDTH - 50, y : LCD_HEIGHT - 70, w : 50, h : 70};
static mut GREEN : Rect = Rect{x : 100, y : 100, w : 25, h : 25};
static mut RED : Rect = Rect{x : 50, y : 50, w : 25, h : 25};
static mut LCD : Lcd = Lcd::new();
static mut RTC : Option<Rtc> = None;
static mut TIM2 : Option<Timer<pac::TIM2>> = None;
static mut TIMER_PAUSE: Option<Timer<pac::TIM1>> = None;

#[allow(dead_code)]
fn game_iter(_tick : u32) {
    let green = unsafe { &mut GREEN };
    let red = unsafe { &mut RED };
    let _blue = unsafe { &mut BLUE };

    let lcd = unsafe { &mut LCD };

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

use tetris_nostd as tetris;
use tetris::*;

const SQUARE_SIZE: usize = 20;
static mut TETRIS : Option<Game> = None;

fn create_tetris() {
    let seed: u64 = 1;
    unsafe { TETRIS = Some(Game::new(seed)) };
}

fn tetris_control() -> Control {
    let mut c = Control::default();

    unsafe { adc::ACCEL_ADC.as_mut().unwrap().start_conversion() };
    let (_, y, x) = unsafe { adc::ACCEL_ADC.as_mut().unwrap().get_axes() };
    let dy = if y < 50 && y > -50 { 0 } else { y };
    let dx = if x < 150 && x > -150 { 0 } else { x };

    if dy > 0 { c.right = true; }
    if dy < 0 { c.left = true; }

    if dx < 0 { c.fall = true; }

    c
}

fn tetris_iter(_tick : u32) {

    let lcd = unsafe { &mut LCD };
    let game = unsafe { TETRIS.as_mut().unwrap() };

    let c = tetris_control();
    game.control(&c);
    game.update();

    for y in 0..BOARD_H {
        for x in 0..BOARD_W {
            let unit = game.get_draw_cell(x, y);
            if unit.dirty
            {
                let color = match unit.state {
                    tetris::CType::Empty  => RGB(0, 0, 0),
                    tetris::CType::Red    => RGB(255, 0, 0),
                    tetris::CType::Blue   => RGB(0, 70, 180),
                    tetris::CType::C1     => RGB(30, 220, 50),
                    tetris::CType::C2     => RGB(255, 210, 0),
                    tetris::CType::C3     => RGB(210, 210, 20),
                    tetris::CType::C4     => RGB(0, 10, 210),
                    _ => panic!("unsupported color"),
                };

                let x_calc = x * SQUARE_SIZE as usize;
                let y_calc = y * SQUARE_SIZE as usize;
                let _ = lcd.fill_rect_with_color(Rect::new(
                    LCD_WIDTH - SQUARE_SIZE - y_calc,
                    x_calc,
                    SQUARE_SIZE as usize,
                    SQUARE_SIZE as usize,
                ), color);
                unit.dirty = false;
            }
        }
    }
}

static mut DBG_TICK : u32 = 0;
//INTERRUPTS
#[interrupt]
fn TIM2() {
    unsafe { TIM2.as_mut().unwrap().clear_update_interrupt_flag() };

    tetris_iter(unsafe { DBG_TICK });
    unsafe { DBG_TICK += 1; }
}

#[interrupt]
fn RTC() {
    let rtc = unsafe { &mut RTC.as_mut().unwrap() };
    rtc.clear_second_flag();

    let _res = unsafe { DBG_TICK };
    unsafe { DBG_TICK = 0; }

    //let _ = rtt_print!("TIM2 hz: {}", res);
}

#[exception]
fn SysTick() {
    static mut DBG_SYSTICK : u32 = 0;
    *DBG_SYSTICK += 1;
}