
use stm32f1xx_hal::pac::{ RCC, ADC3, };
use core::sync::atomic::{AtomicIsize, Ordering};

pub struct Adc {
    x : AtomicIsize,
    y : AtomicIsize,
    z : AtomicIsize,
}

impl Adc {
    pub fn new() -> Self {
        Adc {
            x : AtomicIsize::new(0),
            y : AtomicIsize::new(0),
            z : AtomicIsize::new(0),
        }
    }

    pub fn init(&mut self) {

        let rcc = unsafe { &*RCC::ptr() };

        // ADC Prescaler - PLCK2 divided by 8
        rcc.cfgr.modify(|_, w| w.adcpre().div8());
        // ADC 1 interface clock enable
        rcc.apb2enr.modify(|_, w| w.adc3en().enabled());

        let adc3 = unsafe { &*ADC3::ptr() };

        adc3.cr1.modify(|_, w| { 
            w.scan().enabled()   // Режим сканирования
             .jauto().enabled()  // Автоматическое измерение добавленных каналов
             .jeocie().enabled() // Прервание после окончания измерения добавленных каналов
        });

        adc3.cr2.modify(|_, w| { 
            w.cont().single()
        });

        adc3.sqr3.modify(|_, w| unsafe { 
            w.sq1().bits(4)   // ACCZ    pin 30
        });

        adc3.jsqr.modify(|_, w| unsafe { 
            w.jl().bits(3)    // Количество измерений добавленных каналов = 4
             .jsq1().bits(7)  // PWOUT   pin pf8
             .jsq2().bits(5)  // ACCY    pin 31
             .jsq3().bits(6)  // ACCX    pin 32
             .jsq4().bits(8)  // Current pin G1
        });
    }

    pub fn start_conversion(& self) {

    }

    pub fn callback_conv_ready(& self) {

    }
}