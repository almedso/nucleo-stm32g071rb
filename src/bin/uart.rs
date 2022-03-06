// PA2 (TX) and PA3 (RX) are correct mappted to UART2 -> to STLKINK USB /dev/ttyACM?
// also at stm morpho CN10-6 and CN10-34
// requires solder bridge setting SB16 ON SB18 ON/ requires st link firmware
//
// PC4 - TX - CN9-2
// PC5 - RX - CN9-1 are mapped to USART1
// see https://cdn-reichelt.de/documents/datenblatt/A300/NUCLEO_MANUAL_EN.pdf
// https://ferrous-systems.com/blog/async-on-embedded/

#![no_main]
#![no_std]

use nucleo_stm32g071rb as board;

use nb; // none-blocking crate

use core::fmt::Write;

use board::hal::{prelude::*, serial::*, stm32};

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();
    let gpioc = dp.GPIOC.split(&mut rcc);

    let mut usart1 = dp
        .USART1
        .usart(
            gpioc.pc4,
            gpioc.pc5,
            FullConfig::default()
                .baudrate(115200.bps())
                .fifo_enable()
                .rx_fifo_enable_interrupt()
                .rx_fifo_threshold(FifoThreshold::FIFO_4_BYTES),
            &mut rcc,
        )
        .unwrap();

    writeln!(usart1, "Hello USART1\r\n").unwrap();

    let (mut tx1, mut rx1) = usart1.split();

    let mut cnt = 0;
    loop {
        if rx1.fifo_threshold_reached() {
            loop {
                match rx1.read() {
                    Err(nb::Error::WouldBlock) => {
                        // no more data available in fifo
                        break;
                    }
                    Err(nb::Error::Other(_err)) => {
                        // Handle other error Overrun, Framing, Noise or Parity
                    }
                    Ok(byte) => {
                        writeln!(tx1, "{}: {}\r\n", cnt, byte).unwrap();
                        cnt += 1;
                    }
                }
            }
        }
    }
}
