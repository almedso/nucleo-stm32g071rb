// #![deny(warnings)]
#![no_main]
#![no_std]

use nucleo_stm32g071rb as board;

use cortex_m::{
    self,
    // asm,
    interrupt::{free, Mutex},
};

use core::ops::DerefMut;

use board::hal::{
    gpio::{gpiob::PB3, Input, PullUp},
    interrupt,
    prelude::*,
    stm32,
    timer::Timer,
};

use core::cell::RefCell;
use manchester_code::{BitOrder, Decoder};

static IR_DIODE: Mutex<RefCell<Option<PB3<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));
static DECODER: Mutex<RefCell<Decoder>> = Mutex::new(RefCell::new(Decoder::new(
    true,
    false,
    BitOrder::LittleEndian,
)));
static TIMER_TIM2: Mutex<RefCell<Option<Timer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
unsafe fn TIM2() {
    free(|cs| {
        if let Some(ref mut timer) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() {
            timer.clear_irq();
        }

        if let Some(ref mut ir_diode) = IR_DIODE.borrow(cs).borrow_mut().deref_mut() {
            // let decoder = DECODER.borrow(cs).borrow_mut().deref_mut();
            match DECODER
                .borrow(cs)
                .borrow_mut()
                .deref_mut()
                .next(ir_diode.is_high().unwrap())
            {
                None => (),
                Some(t) => {
                    if t.len() > 2 {
                        defmt::println!("Datagram: {:?}", t);
                    }
                }
            };
        }
    });
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();

    let gpiob = dp.GPIOB.split(&mut rcc);
    let ir_diode = gpiob.pb3.into_pull_up_input();

    let mut timer = dp.TIM2.timer(&mut rcc);
    timer.start(296.us()); // 889 µs / 4; aka 4 samples per half bit period
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
        IR_DIODE.borrow(cs).replace(Some(ir_diode));
    });

    defmt::println!("Start receiving ... (little endian)");

    loop {
        // asm::wfi();  // does not work in conjunction with  rtt defmt
    }
}
