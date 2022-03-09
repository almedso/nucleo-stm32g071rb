# Overview

This is a no-std project providing a

*  **Nucleo-STM32G071RB**  evaluation board
* defmt based rtt logging
* stack overflow protection

as base to run application (located in *bin* directory) on


# Setup

## Exta tool setup

This project uses [`probe-run`] + [`defmt`] + [`flip-link`] embedded project

[`probe-run`]: https://crates.io/crates/probe-run
[`defmt`]: https://github.com/knurling-rs/defmt
[`flip-link`]: https://github.com/knurling-rs/flip-link

as extra tooling. Follow the links above to understand how to setup those tools
in addition to cargo.

## Board / Cross compile setup

### .cargo/config.toml

contains target settings, linker settings, probe-run settings


### target support

Add the target with `rustup`.

``` console
$ rustup target add thumbv6m-none-eabi
```

### HAL as a dependency

Since we use Nucleo-STM32G071RB we use the [`stm32g0xx-hal`].

[`stm32g0xx-hal`]: https://crates.io/crates/stm32g0xx-hal

### Linker Script

Our hal requires a linker script. It is `memory.x`


If you're running out of memory (`flip-link` bails with an overflow error),
you can decrease the size of the device memory buffer by setting
the `DEFMT_RTT_BUFFER_SIZE` environment variable.
The default value is 1024 bytes, and powers of two should be used for optimal performance:

``` console
$ DEFMT_RTT_BUFFER_SIZE=64 cargo run --bin hello
```

### Probe run


Check which defmt version `probe-run` supports

``` console
$ probe-run --version
0.2.0 (aa585f2 2021-02-22)
supported defmt version: 60c6447f8ecbc4ff023378ba6905bcd0de1e679f
```

# License

Licensed under MIT license [LICENSE-MIT](LICENSE-MIT) 

# Hardware references

* [User Guide for Nucleo-STM32G071RB](https://www.st.com/resource/en/user_manual/um2324-stm32-nucleo64-boards-mb1360-stmicroelectronics.pdf)
