#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use nucleo_stm32g071rb as board;

use board::hal::nb::block;
use board::hal::prelude::*;
use board::hal::stm32;

use manchester_code::{BitOrder, Decoder, SyncOnTurningEdge, ActivityLevel};

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();

    let gpiob = dp.GPIOB.split(&mut rcc);
    // let infrared = gpiob.pb3.into_pull_up_input();
    let infrared = gpiob.pb3.into_floating_input();

    let mut timer = dp.TIM17.timer(&mut rcc);
    timer.start(296.us()); // 889 µs / 4; aka 4 samples per half bit period
    let mut receiver = Decoder::new(
        ActivityLevel::Low,
        SyncOnTurningEdge::Second,
        BitOrder::BigEndian,
    );
    defmt::println!("Start receiving ... (big endian)");

    loop {
        match receiver.next(infrared.is_high().unwrap()) {
            None => (),
            Some(t) => {
                if t.len() > 2 {
                    defmt::println!("Datagram: {:?}", t);
                }
            }
        };
        block!(timer.wait()).unwrap();
    }
}
