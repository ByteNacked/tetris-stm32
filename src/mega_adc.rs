#![allow(dead_code)]

use crate::pld::{PldSpi, PldRdSpi, PldFifo, PldAdcMode, PldCfg, PldGie, PldIe};
use crate::pac::DWT;
use crate::rtt_print;

pub struct MegaAdc { 
    cnt : u16,
}

static mut ADC_FRAME : [i32;0x10] = [0;0x10];

impl MegaAdc {
    pub fn new() -> Self {
        MegaAdc { cnt : 0 }
    }

    pub fn init(&mut self) {
        cmd::adc_stop();
        busy_wait_cycles!(72000 * 40);

        cmd::write(&tables::ADC1298_STOP_CMD);

        PldRdSpi::get().write(0x0018); // Установка 32000 Гц реограф
        PldFifo::get().write(0x0018);

        PldAdcMode::get().write(0x0002); // Режим программирования АЦП
        busy_wait_cycles!(72000 * 5);

        PldCfg::get().write(0x0051); // Сброс FIFO и блоков АЦП
        busy_wait_cycles!(72000 * 1);

        PldCfg::get().write(0x0032); // Снять сброс с FIFO	+ режим программного опроса
        busy_wait_cycles!(72000 * 1);

        // Сброс ADC
        {
            cmd::adc_reset_on();
            busy_wait_cycles!(72000 * 10);
            cmd::adc_reset_off();
        }

        cmd::write(&tables::ADC1298_STOP_CMD); // Опять?
        cmd::write(&tables::ADC1298_SET1KHZ_CMD);
        cmd::write(&tables::ADC1298_SIGNAL_CMD);
        cmd::write(&tables::ADC1298_START_CMD);

        // TODO: read-out tests

        PldIe::get().write(0x0008); // Разрешение выдачи прерывания FIFO_AEMPTY на ногу RSV1 - нужно читать 41 слово
        PldGie::get().write(0x0001); // Глобальное разрешение прерываний внутри PLD

        //TODO: init Afe here

        PldAdcMode::get().write(0x0000); // Принимаем данные из АЦП в PLD, рабочий режим // 0 - работа с FIFO. 4 - работа с SPI
        busy_wait_cycles!(72000 * 1);

        cmd::adc_start(); // Поехали!
    }

    pub fn try_samples(&mut self) {
        while cmd::is_adc_rdy() {
            match self.try_read_adc1298_fifo() {
                Ok(_) => (), //rtt_print!("cnt : {}", self.cnt),
                Err(_) => (), //rtt_print!("Adc fifo err"),
            }
        }
    }

    pub fn try_read_adc1298_fifo(&mut self) -> Result<(),()> {
        
        // Вычитываем заголовок
        let head = PldFifo::get().read(); // голова проверка ниже по коду
        let _dummy = PldFifo::get().read(); // пустые вычитывания статусов
        let _dummy = PldFifo::get().read();
        let _dummy = PldFifo::get().read();
        let status_l : u32 = PldFifo::get().read() as u32; // младшая часть статусного рега
        let status_h : u32 = PldFifo::get().read() as u32; // старшая
        let _status : u32 = (status_h << 16) | status_l;

        if (head ^ 0xFEFE) != 0 { 
            rtt_print!("head");
            Err(())? 
        } // тест корректности головы

        // Вычитываем точки
        for _ in 0 .. 16/2 {
            let bb_l = PldFifo::get().read() as u32;
            let bb_h = PldFifo::get().read() as u32;
            let _bb : u32 = (bb_h << 16) | bb_l; 

            let bb_l = PldFifo::get().read() as u32;
            let bb_h = PldFifo::get().read() as u32;
            let _bb : u32 = (bb_h << 16) | bb_l;

            //TODO: processing
        }

        // Вычитываем хвост
        let _dummy = PldFifo::get().read(); // чтение CRC
        let cnt = PldFifo::get().read(); // счётчик
        let _tail = PldFifo::get().read(); // Хвост 0xBDBD читаем но не проверяем

        if cnt != self.cnt { 
            rtt_print!("{} != {}", cnt, self.cnt);
            self.cnt = cnt.wrapping_add(1);
            Err(())? 
        } // контроль счетчика

        self.cnt = cnt.wrapping_add(1);
        // TODO: write cnt to output frame

        Ok(())
    }

}

#[rustfmt::skip]
mod cmd {
    use stm32f1xx_hal::pac::{GPIOC, GPIOB, GPIOE, DWT};
    use super::PldSpi;
    
    pub fn is_adc_rdy() -> bool {
        let gpioe = unsafe { &*GPIOE::ptr() };
        gpioe.idr.read().idr0().is_low()
    }

    pub fn adc_reset_on() {
        let gpiob = unsafe { &*GPIOB::ptr() };
        gpiob.odr.modify(|_, w| w.odr13().low());
    }

    pub fn adc_reset_off() {
        let gpiob = unsafe { &*GPIOB::ptr() };
        gpiob.odr.modify(|_, w| w.odr13().high());
    }

    pub fn adc_start() {
        let gpioc = unsafe { &*GPIOC::ptr() };
        gpioc.odr.modify(|_, w| w.odr7().high());
    }

    pub fn adc_stop() {
        let gpioc = unsafe { &*GPIOC::ptr() };
        gpioc.odr.modify(|_, w| w.odr7().low());
    }

    pub fn write(cmd : &[u16]) {
        for i in cmd { data_send(*i); }
    }

    fn data_send(b : u16) {
        PldSpi::get().write(b);
        busy_wait_cycles!(72000 * 1);
    }
}

#[rustfmt::skip]
mod tables {
    pub const ADC1298_START_CMD : [u16;3]  = [ 0x2200,0x1110,0x6600 ]; // Команда Enable Data Continious Mode - 0x10
    pub const ADC1298_STOP_CMD : [u16;3]   = [ 0x2200,0x1111,0x6600 ]; // Команда Stop Data Continious Mode - 0x11

    pub const ADC1298_SET1KHZ_CMD : [u16;5]  = [ 0x2200,0x1141,0x1100,0x1104,0x6600 ]; // Запись частот 1 КГц - @(0x4) Никакая другая частота невозможна
    pub const ADC1298_GETKHZ_CMD : [u16;5]   = [ 0x2200,0x1121,0x1100,0x1100,0x6600 ]; // Чтение регистра частот

    // Инициализация для внешнего сигнала
    pub const ADC1298_SIGNAL_CMD : [u16;140] = 
    [ 
        0x0200,0x0142,0x0100,0x0115,0x0600,		// Перевод тестового сигнала на внутренний источник - @(0x10)
        0x0200,0x0143,0x0100,0x01C0,0x0600,		// Использовать внутреннюю опору
        0x0200,0x0145,0x0100,0x0110,0x0600,		// REO			
        0x0200,0x0146,0x0100,0x0110,0x0600,		// Gain для канала 1 = 1 для АЦП 0		- @(0x6) <- 0x50		
        0x0200,0x0147,0x0100,0x0110,0x0600,		// Gain для канала 2 = 1 для АЦП 0		- @(0x7) <- 0x50		
        0x0200,0x0148,0x0100,0x0110,0x0600,		// Gain для канала 3 = 1 для АЦП 0		- @(0x8) <- 0x50
        0x0200,0x0149,0x0100,0x0160,0x0600,		// Gain для канала 4 = 1 для АЦП 0		- @(0x9) <- 0x50	в 12 раз канал Микрофона			
        0x0200,0x014A,0x0100,0x0110,0x0600,		// Gain для канала 5 = 1 для АЦП 0		- @(0xa) <- 0x50		
        0x0200,0x014B,0x0100,0x0110,0x0600,		// Gain для канала 6 = 1 для АЦП 0		- @(0xb) <- 0x50		
        0x0200,0x014C,0x0100,0x0150,0x0600,		// ECG - C6F

        0x2000,0x1042,0x1000,0x1015,0x6000,		// Перевод тестового сигнала на внутренний источник - @(0x10)
        0x2000,0x1043,0x1000,0x10C0,0x6000,		// Использовать внутреннюю опору
        0x2000,0x1045,0x1000,0x1010,0x6000,		// REO
        0x2000,0x1046,0x1000,0x1050,0x6000,		// ECG
        0x2000,0x1047,0x1000,0x1050,0x6000,		// ECG
        0x2000,0x1048,0x1000,0x1050,0x6000,		// ECG
        0x2000,0x1049,0x1000,0x1050,0x6000,		// ECG
        0x2000,0x104A,0x1000,0x1050,0x6000,		// ECG
        0x2000,0x104B,0x1000,0x1050,0x6000,		// ECG
        0x2000,0x104C,0x1000,0x1050,0x6000,		// ECG
        
        // Отключены рео настройки по просьбе Севы, 03.10.2019
        //	0x0200,0x0156,0x0100,0x01A3,0x0600,		// Включить внутренний демодулятор, использовать внешний модулятор
        //	0x2000,0x1056,0x1000,0x10A3,0x6000,		// Включить внутренний демодулятор, использовать внешний модулятор
        
        0x2000,0x1044,0x1000,0x100F,0x6000,   // Включить подтяжки
        0x2000,0x104F,0x1000,0x10F6,0x6000, //LOFFP	// Это АЦП отвечает за ЭКГ
        0x2000,0x1050,0x1000,0x1080,0x6000, //LOFFN
        0x2000,0x1051,0x1000,0x1034,0x6000, //FLIP	
        
        0x0200,0x0144,0x0100,0x010F,0x0600,   // Включить подтяжки
        0x0200,0x014F,0x0100,0x0180,0x0600, //LOFFP	// В этом АЦП только один ЭКГ канал
        0x0200,0x0150,0x0100,0x0180,0x0600, //LOFFN
        0x0200,0x0151,0x0100,0x0180,0x0600, //FLIP	
    ];

	// Таблица сдвига для каналов, оставляем только нужное количество разрядов
    pub const AD1298_SHIFT : [usize; 16] = 
    [
        (24-22),// int reo0;		// 21 рео
        (24-22),// int reo1;		// 21 рео
        (24-16),// int agnd;		
        (24-19),// int ch4;			// 19 экг
        (24-16),// int agnd;
        (24-19),// int ch2;			// 19 экг
        (24-16),// int agnd;	
        (24-19),// int chF;			// 19 экг
        (24-16),// int mic1;		// микрофон канала АД
        (24-19),// int ch5;			// 19 экг
        (24-20),// int ad;			// давление
        (24-19),// int ch3;			// 19 экг
        (24-16),// int brth;		// канал APT - дыхание и храп с датчика давления
        (24-19),// int ch1;			// 19 экг
        (24-19),// int ch6;			// 19 экг
        (24-19),// int chL;			// 19 экг
    ];

    // Таблица перестановки каналов, для того, чтобы дальше использовать по порядку 
    pub const AD1298_CHANGE_12X10 : [usize; 17] = 
    [ 
        8,// int reo0;
        9,// int reo1;		// 19 экг
        13,// int agnd;		// Сюда нужно засунуть RED
        5,// int ch4;		// 19 экг
        14,// int agnd;		// Сюда нужно засунуть IR
        3,// int ch2;		// 19 экг
        15,// int agnd;		// 21 рео
        1,// int chF;		// 19 экг
        12,// int mic1;
        6,// int ch5;		// 19 экг
        11,// int ad;
        4,// int ch3;		// 19 экг
        10,// int brth;
        2,// int ch1;		// 19 экг
        7,// int ch6;
        0,// int chL;		// 19 экг
        16,// int Count
    ];
}