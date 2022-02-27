#![no_main]
#![no_std]

use nucleo_stm32g071rb as board;

use cortex_m::{
    self,
    // asm,
    interrupt::{free, Mutex},
};

use board::hal::{
    interrupt,
    prelude::*,
    stm32::{self, TIM3},
    timer::{pwm::PwmPin, Channel1, Timer},
};
pub use embedded_hal::prelude::*;

use core::cell::RefCell;
use core::ops::DerefMut;

use manchester_code::{Datagram, DatagramBigEndianIterator, Encoder};

static IR_EMITTER: Mutex<RefCell<Option<PwmPin<TIM3, Channel1>>>> = Mutex::new(RefCell::new(None));
static ENCODER: Mutex<RefCell<Option<Encoder<DatagramBigEndianIterator>>>> =
    Mutex::new(RefCell::new(None));
static TIMER_TIM2: Mutex<RefCell<Option<Timer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
unsafe fn TIM2() {
    free(|cs| {
        if let Some(ref mut timer) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() {
            timer.clear_irq();
        }
        // Do not do anything if the infrared emitter is not configured (yet)
        if let Some(ref mut ir_emitter) = IR_EMITTER.borrow(cs).borrow_mut().deref_mut() {
            // Check if we do have an encoder active
            let encoder_mutex = ENCODER.borrow(cs);
            if let Some(ref mut encoder) = encoder_mutex.borrow_mut().deref_mut() {
                match encoder.next() {
                    Some(half_bit) => {
                        if half_bit {
                            ir_emitter.enable();
                        } else {
                            ir_emitter.disable();
                        }
                    }
                    None => {
                        ir_emitter.disable();
                        encoder_mutex.replace(None); // remove the encoder since datagram is sent
                    }
                }
            }
        }
    });
}

#[allow(clippy::empty_loop)]
#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");

    // Set up the system clock
    //// let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Setup PWM we use Arduino PIN D5 -> is PB4 / TIM3_CH1 on stm32g071
    let gpiob = dp.GPIOB.split(&mut rcc);

    // let clocks = rcc.cfgr.sysclk(16.MHz()).freeze(&mut flash.acr);

    let pwm_pin = gpiob.pb4;
    let pwm = dp.TIM3.pwm(36_u32.khz(), &mut rcc);
    let mut pwm_send_ir = pwm.bind_pin(pwm_pin);

    pwm_send_ir.set_duty(pwm_send_ir.get_max_duty() / 4); // 25% duty cycle

    // Set up the interrupt timer
    let mut timer = dp.TIM2.timer(&mut rcc);
    timer.start(889.us());
    timer.listen();

    // Enable interrupt
    stm32::NVIC::unpend(interrupt::TIM2);
    unsafe {
        stm32::NVIC::unmask(interrupt::TIM2);
    }

    // Move shared resources to Mutex
    // free wraps a critical section see https://docs.rs/cortex-m/0.7.4/cortex_m/interrupt/fn.free.html
    free(|cs| {
        TIMER_TIM2.borrow(cs).replace(Some(timer));
        IR_EMITTER.borrow(cs).replace(Some(pwm_send_ir));
    });

    defmt::println!("Init done");

    // set a datagram to send
    let datagram = Datagram::new("0101_0011_0111_0001");
    defmt::println!("Send new datagram {}", datagram);

    let mut delay = dp.TIM15.delay(&mut rcc);

    loop {
        delay.delay(1000.ms());

        defmt::println!("Send new da agram {}", datagram);

        free(|cs| {
            let encoder_mutex = ENCODER.borrow(cs);
            if encoder_mutex.borrow_mut().deref_mut().is_none() {
                // only inject a new datagram if no datagram is sent anymore
                let encoder = Encoder::<DatagramBigEndianIterator>::new(datagram);
                encoder_mutex.replace(Some(encoder));
            }
        });
    }
}
