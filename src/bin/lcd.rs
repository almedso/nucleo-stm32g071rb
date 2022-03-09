#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use nucleo_stm32g071rb as board; //  it also includes mem, defmt

use board::lcd::{Color, RgbLCD};

use board::hal::i2c::Config;
use board::hal::prelude::*;
use board::hal::stm32;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Startup");

    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();
    let delay = dp.TIM15.delay(&mut rcc);

    let gpiob = dp.GPIOB.split(&mut rcc);

    let sda = gpiob.pb9.into_open_drain_output();
    let scl = gpiob.pb8.into_open_drain_output();

    let mut i2c = dp.I2C1.i2c(
        sda,
        scl,
        // Config::with_timing(0x2020_151b),
        100.khz(),
        &mut rcc,
    );

    defmt::info!("I2C initialized");

    let mut lcd = RgbLCD::new();
    lcd.init(&mut i2c, delay).unwrap();

    defmt::info!("LCD initialized");

    defmt::info!("LCD switch on backlight");
    lcd.switch_display_off(&mut i2c).unwrap();
    lcd.set_color(&mut i2c, Color::Blue).unwrap();
    lcd.write_byte(&mut i2c, b'R').unwrap();
    defmt::info!("Write R");

    // loop {
    //     match i2c.write(0x3c, &buf) {
    //         Ok(_) => hprintln!("ok").unwrap(),
    //         Err(err) => hprintln!("error: {:?}", err).unwrap(),
    //     }
    // }
    nucleo_stm32g071rb::exit()
}
