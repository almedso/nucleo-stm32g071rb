#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use nucleo_stm32g071rb as board;

use board::hal::prelude::*;
use board::hal::stm32;

use rotary_encoder_hal::{Direction, Rotary};

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let cp = cortex_m::Peripherals::take().expect("cannot take core peripherals");

    let mut rcc = dp.RCC.constrain();
    let mut delay = cp.SYST.delay(&mut rcc);

    let gpioc = dp.GPIOC.split(&mut rcc);
    let gpioa = dp.GPIOA.split(&mut rcc);
    let b1 = gpioc.pc7.into_pull_up_input();
    let b2 = gpioa.pa9.into_pull_up_input();

    defmt::info!("Buttons Initialized");
    // let mut b1_state: bool = b1.is_high().unwrap();
    // let mut b2_state: bool = b2.is_high().unwrap();
    // defmt::info!("B1: {} B2: {}", b1_state, b2_state);

    let mut enc = Rotary::new(b1, b2);

    loop {
        match enc.update().unwrap() {
            Direction::Clockwise => {
                defmt::info!("Clockwise");
            }
            Direction::CounterClockwise => {
                defmt::info!("Counterclockwise");
            }
            Direction::None => {}
        }
        delay.delay(10.ms());
    }
    // loop {
    //     let b1_state_new: bool = b1.is_high().unwrap();
    //     let b2_state_new: bool = b2.is_high().unwrap();
    //     delay.delay(100.ms());
    //     if (   b1_state_new && ! b1_state)
    //     || ( ! b1_state_new &&   b1_state)
    //     || (   b2_state_new && ! b2_state)
    //     || ( ! b2_state_new &&   b2_state) {
    //         b1_state = b1_state_new;
    //         b2_state = b2_state_new;
    //         defmt::info!("B1: {} B2: {}", b1_state, b2_state);
    //     }
    // }
}
