#![no_std]
#![cfg_attr(test, no_main)]

use nucleo_stm32g071rb as _; // memory layout + panic handler

#[defmt_test::tests]
mod tests {}
