#![no_std]
#![no_main]
#![feature(asm)]
#![feature(generators, generator_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(const_fn)]
#![feature(never_type)]
#![allow(unused_imports)]
#![allow(incomplete_features)]
#![allow(dead_code)]

extern crate jlink_rtt;
extern crate panic_rtt;

#[macro_use]
mod macros;
mod adc;
mod beeper;
mod button;
mod cmd;
mod debug;
mod embbox;
mod lcd;
mod mega_adc;
mod pause;
mod pld;
mod port;
mod sche;
mod splash;
mod tps;
mod usb;
mod cb;

use core::fmt::Binary;
use core::panic::PanicInfo;
use cortex_m::asm;
use cortex_m::asm::{bkpt, delay, wfi};
use cortex_m::interrupt::{disable as int_disable, enable as int_enable};
use cortex_m_rt::{entry, exception};
use embbox::EmbBox;
use embedded_hal::digital::v2::OutputPin;
pub(crate) use jlink_rtt::rtt_print;
use lcd::{color::*, Lcd, Rect, FULL_SCREEN_RECT, LCD_HEIGHT, LCD_WIDTH};
use mega_adc::{ MegaAdc, AdcFrame, AfeFrame};
use nb::block;
use pause::pause;
use port::*;
use stm32f1xx_hal::{
    afio, i2c, pac,
    prelude::*,
    rtc::Rtc,
    stm32,
    stm32::interrupt,
    stm32::Interrupt,
    time,
    timer::{Event, Timer},
};
use tps::Tps;
use cb::CircularBuffer;

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

    let mut nvic = cp.NVIC;
    //nvic.enable(Interrupt::USB_LP_CAN_RX0);

    let clocks = rcc.cfgr.freeze_72Mhz_usb(&mut flash.acr);

    unsafe { MONO_TIMER = Some(time::MonoTimer::new(cp.DWT, clocks)) };

    let mut gpiob = unsafe { dp.GPIOB.steal() };
    let mut gpiog = unsafe { dp.GPIOG.steal() };

    port_init();
    fsmc_init();

    let mut led_red = gpiog.pg6.into_push_pull_output(&mut gpiog.crl);
    let mut led_green = gpiog.pg7.into_push_pull_output(&mut gpiog.crl);

    let mut pwr = dp.PWR;
    let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut rcc.apb1, &mut pwr);

    unsafe { RTC = Some(Rtc::rtc(dp.RTC, &mut backup_domain)) };
    let rtc = unsafe { &mut RTC.as_mut().unwrap() };
    rtc.listen_seconds();
    nvic.enable(Interrupt::RTC);

    let mut systick = Timer::syst(cp.SYST, 100.hz(), clocks);
    systick.listen(Event::Update);

    unsafe { TIMER_PAUSE = Some(Timer::tim1(dp.TIM1, 1.hz(), clocks, &mut rcc.apb2)) };

    unsafe { adc::ACCEL_ADC = Some(adc::Adc::new()) };
    let adc = unsafe { adc::ACCEL_ADC.as_mut().unwrap() };
    adc.init();
    adc.start_conversion();

    let lcd = unsafe { &mut LCD };

    lcd.init();

    splash::draw_sreens(lcd);

    unsafe { TIM2 = Some(Timer::tim2(dp.TIM2, 15.ms(), clocks, &mut rcc.apb1)) };
    let tim2 = unsafe { TIM2.as_mut().unwrap() };
    tim2.listen(Event::Update);
    unsafe { nvic.set_priority(Interrupt::TIM2, 3 << 4) };

    create_tetris();
    nvic.enable(Interrupt::TIM2);
    usb::usb_init(&clocks);

    // I2C1

    let mut pb8 = gpiob.pb8.into_push_pull_output(&mut gpiob.crh);
    let mut pb9 = gpiob.pb9.into_push_pull_output(&mut gpiob.crh);

    //for _ in 0 .. 10 {
    //    pb8.set_low();
    //    pb8.set_low();
    //    pause(10.ms());

    //    pb9.set_high();
    //    pb9.set_high();
    //    pause(10.ms());
    //}

    let i2c1_pins = (
        pb8.into_alternate_open_drain(&mut gpiob.crh),
        pb9.into_alternate_open_drain(&mut gpiob.crh),
    );
    let mut afio_mapr = dp.AFIO.constrain(&mut rcc.apb2).mapr;

    use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
    let mut i2c1 = i2c::BlockingI2c::i2c1(
        dp.I2C1,
        i2c1_pins,
        &mut afio_mapr,
        i2c::Mode::Fast {
            frequency: 400_000,
            duty_cycle: i2c::DutyCycle::Ratio16to9,
        },
        clocks,
        &mut rcc.apb1,
        10_000,
        10,
        10_000,
        10_000,
    );

    let mut i2c1_buf = [0u8; 1];
    rtt_print!("{:?}", i2c1.write(0x90 >> 1, &[0x01]));
    rtt_print!("{:?}", i2c1.read(0x90 >> 1, &mut i2c1_buf));
    rtt_print!("{:b}", i2c1_buf[0]);

    rtt_print!("{:?}", i2c1.write_read(0x90 >> 1, &[0x01], &mut i2c1_buf));
    rtt_print!("{:b}", i2c1_buf[0]);

    unsafe {
        I2C1_DRIVER = Some(i2c1);
    }
    let i2c1_ref: &'static mut I2C1Driver;
    unsafe {
        i2c1_ref = I2C1_DRIVER.as_mut().unwrap();
    }

    //TPS
    let mut tps = tps::Tps::new(i2c1_ref);
    let _ = tps.init();

    //PLD
    pld::pld_init();

    //MEGA_ADC
    let mega_adc = unsafe { &mut MEGA_ADC };
    mega_adc.init();

    loop {
        read_from_mega_adc();
        //pause(500.ms());
        //let _ = led_green.set_high();
        //let _ = led_red.set_low();
        //pause(500.ms());
        //let _ = led_green.set_low();
        //let _ = led_red.set_high();
    }
}

fn read_from_mega_adc() {
    cortex_m::interrupt::free( |_cs| {
        let mega_adc = unsafe { &mut MEGA_ADC };
        let vb = unsafe { &mut VB };

        mega_adc.try_samples(&mut |adc, afe| {
            use core::mem::size_of;

            if vb.is_full() {
                rtt_print!("VB overflow!");
            }

            let adc_cnt = adc.cnt;
            if adc_cnt % 1000 == 0 {
                rtt_print!("{:?}", adc);
            }

            vb.enqueue(0x7E);
            vb.enqueue(size_of::<AdcFrame>() as u8);
            vb.enqueue(0x00);
            vb.enqueue(0x81);
            vb.enqueue_slice(adc.as_bytes());
            vb.enqueue(0xFF);
            vb.enqueue(0xFF);
            vb.enqueue(0xBD);

            //vb.enqueue(0x7E);
            //vb.enqueue(size_of::<AfeFrame>() as u8);
            //vb.enqueue(0x00);
            //vb.enqueue(0x82);
            //vb.enqueue_slice(afe.as_bytes());
            //vb.enqueue(0xFF);
            //vb.enqueue(0xFF);
            //vb.enqueue(0xBD);
        });       
    });
}

static mut BLUE: Rect = Rect {
    x: LCD_WIDTH - 50,
    y: LCD_HEIGHT - 70,
    w: 50,
    h: 70,
};
static mut GREEN: Rect = Rect {
    x: 100,
    y: 100,
    w: 25,
    h: 25,
};
static mut RED: Rect = Rect {
    x: 50,
    y: 50,
    w: 25,
    h: 25,
};

static mut LCD: Lcd = Lcd::new();
static mut RTC: Option<Rtc> = None;
static mut TIM2: Option<Timer<pac::TIM2>> = None;
static mut TIMER_PAUSE: Option<Timer<pac::TIM1>> = None;
static mut MAGIC_NUM: Option<u32> = None;
static mut MONO_TIMER: Option<time::MonoTimer> = None;

type I2C1Driver = stm32f1xx_hal::i2c::BlockingI2c<
    pac::I2C1,
    (
        stm32f1xx_hal::gpio::gpiob::PB8<
            stm32f1xx_hal::gpio::Alternate<stm32f1xx_hal::gpio::OpenDrain>,
        >,
        stm32f1xx_hal::gpio::gpiob::PB9<
            stm32f1xx_hal::gpio::Alternate<stm32f1xx_hal::gpio::OpenDrain>,
        >,
    ),
>;

static mut I2C1_DRIVER: Option<I2C1Driver> = None;
static mut TPS: Option<Tps<I2C1Driver>> = None;
static mut MEGA_ADC: MegaAdc = MegaAdc::new();
static mut VB : CircularBuffer = CircularBuffer::new();

#[allow(dead_code)]
fn game_iter(_tick: u32) {
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

    let yy: isize = red.y as isize + (dy / 8);
    red.y = if yy + red.h as isize > LCD_HEIGHT as isize {
        LCD_HEIGHT - red.h
    } else if yy < 0 {
        0
    } else {
        yy as usize
    };

    let xx: isize = green.x as isize + 20 * dx.signum();
    green.x = if xx + green.w as isize > LCD_WIDTH as isize {
        LCD_WIDTH - green.w
    } else if xx < 0 {
        0
    } else {
        xx as usize
    };

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

use tetris::*;
use tetris_nostd as tetris;

const SQUARE_SIZE: usize = 20;
static mut TETRIS: Option<Game> = None;

fn create_tetris() {
    let seed: u64 = 1;
    unsafe { TETRIS = Some(Game::new(seed)) };
}

fn tetris_control(tick: u32) -> Control {
    static mut ACCUM_Y: i32 = 0;
    static mut ACCUM_X: i32 = 0;
    static mut LVL_RET: bool = true;
    let mut c = Control::default();
    let accum_y = unsafe { &mut ACCUM_Y };
    let accum_x = unsafe { &mut ACCUM_X };

    // panic test
    //    let _ = unsafe { MAGIC_NUM.expect(dbg_info!()) };

    unsafe { adc::ACCEL_ADC.as_mut().unwrap().start_conversion() };
    let (_, y, x) = unsafe { adc::ACCEL_ADC.as_mut().unwrap().get_axes() };

    if x > -100 {
        unsafe {
            LVL_RET = true;
        }
    }

    let dx = if x.abs() < 250 { 0 } else { x };
    let dy = if y.abs() < 40 { 0 } else { y };

    if tick % 30 == 0 {
        //rtt_print!("dy : {}, dx : {}", dy, dx);
    }

    {
        if dy.abs() > 150 {
            *accum_y += dy as i32 * 6;
        } else {
            *accum_y += dy as i32;
        }

        const L1: i32 = 1300;
        if *accum_y > L1 {
            c.right = true;
            *accum_y -= L1;
        } else if *accum_y < -L1 {
            c.left = true;
            *accum_y += L1;
        }
    }
    {
        //accum_x.saturating_add(dx as i32);
        //if unsafe {LVL_RET} && accum_x.abs() > 15000 {
        //    c.fall = true;
        //    *accum_x = 0;
        //    unsafe { LVL_RET = false;}
        //}

        if unsafe { LVL_RET } && dx < -350 {
            c.fall = true;
            unsafe {
                LVL_RET = false;
            }
        }
    }

    c
}

fn tetris_iter(tick: u32) {
    let now = unsafe { MONO_TIMER.as_ref().unwrap().now() };

    let lcd = unsafe { &mut LCD };
    let game = unsafe { TETRIS.as_mut().unwrap() };

    let c = tetris_control(tick);
    game.control(&c);
    game.update();

    for y in 0..BOARD_H {
        for x in 0..BOARD_W {
            let unit = game.get_draw_cell(x, y);
            if unit.dirty {
                let color = match unit.state {
                    tetris::CType::Empty => RGB(0, 0, 0),
                    tetris::CType::Red => RGB(255, 0, 0),
                    tetris::CType::Blue => RGB(0, 70, 180),
                    tetris::CType::C1 => RGB(30, 220, 50),
                    tetris::CType::C2 => RGB(255, 210, 0),
                    tetris::CType::C3 => RGB(210, 210, 20),
                    tetris::CType::C4 => RGB(0, 10, 210),
                    _ => panic!("unsupported color"),
                };

                let x_calc = x * SQUARE_SIZE as usize;
                let y_calc = y * SQUARE_SIZE as usize;
                let _ = lcd.fill_rect_with_color(
                    Rect::new(
                        LCD_WIDTH - SQUARE_SIZE - y_calc,
                        x_calc,
                        SQUARE_SIZE as usize,
                        SQUARE_SIZE as usize,
                    ),
                    color,
                );
                unit.dirty = false;
            }
        }
    }
    //PERF
    //if tick % 60 == 0 {
    //    let msr_tick = now.elapsed() as f32;
    //    let frq = unsafe { MONO_TIMER.as_ref().unwrap().frequency().0 } as f32;
    //    let sec = 1000f32 / frq * msr_tick;
    //    rtt_print!("PERF: tick : {}, sec : {}", msr_tick, sec);
    //}
}

static mut DBG_TICK: u32 = 0;
//INTERRUPTS
#[interrupt]
fn TIM2() {
    unsafe { TIM2.as_mut().unwrap().clear_update_interrupt_flag() };

    tetris_iter(unsafe { DBG_TICK });
    unsafe {
        DBG_TICK += 1;
    }
}

#[interrupt]
fn RTC() {
    let rtc = unsafe { &mut RTC.as_mut().unwrap() };
    rtc.clear_second_flag();

    let _res = unsafe { DBG_TICK };
    unsafe {
        DBG_TICK = 0;
    }

    //let _ = rtt_print!("TIM2 hz: {}", res);
}

#[exception]
fn SysTick() {
    static mut DBG_SYSTICK: u32 = 0;
    *DBG_SYSTICK += 1;

    unsafe {
        button::BUTTON_LEFT.update_state_100hz();
        button::BUTTON_RIGHT.update_state_100hz();
    }
}
