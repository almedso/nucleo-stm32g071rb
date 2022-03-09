//!
//! Port of LCD RGB-Backlight cpp library from grove at github
//! https://github.com/Seeed-Studio/Grove_LCD_RGB_Backlight
//!
//! Datasheets:
//!
//! * 4 bit Fm + i2c bus LED driver
//!   https://files.seeedstudio.com/wiki/Grove_LCD_RGB_Backlight/res/PCA9633.pdf
//! * Seeed pcb specification
//!   https://files.seeedstudio.com/wiki/Grove_LCD_RGB_Backlight/res/JHD1313%20FP-RGB-1%201.4.pdf
//! * LCD controller AIP31068L (16 character x 2 line)

use embedded_hal as hal;

use hal::blocking::delay::DelayUs;
use hal::blocking::i2c::Write;

pub enum Color {
    White,
    Red,
    Green,
    Blue,
    RGB(u8, u8, u8),
}

pub struct RgbLCD {
    display_function: u8,
    display_control: u8,
    display_mode: u8,
}

// const LCD_ADDRESS: u8 = 0x7c >> 1;
// const RGB_ADDRESS: u8 = 0xc4 >> 1;
const LCD_ADDRESS: u8 = 0x3e;
const RGB_ADDRESS: u8 = 0x62;

// Mask's for LCD commands
const LCD_ENTRY_MODESET: u8 = 0x04;
const LCD_DISPLAY_CONTROL: u8 = 0x08;
const LCD_CURSOR_SHIFT: u8 = 0x10;
const LCD_FUNCTION_SET: u8 = 0x20;
const LCD_SET_CGRAM_ADDR: u8 = 0x40;
#[allow(dead_code)]
const LCD_SET_DDRAM_ADDR: u8 = 0x80;

// flags for display entry mode
const LCD_ENTRY_RIGHT: u8 = 0x00;
const LCD_ENTRY_LEFT: u8 = 0x02;
const LCD_ENTRY_SHIFT_INCREMENT: u8 = 0x01;
const LCD_ENTRY_SHIFT_DECREMENT: u8 = 0x00;

// flags for display/cursor shift
const LCD_DISPLAY_MOVE: u8 = 0x08;
#[allow(dead_code)]
const LCD_CURSOR_MOVE: u8 = 0x00;
const LCD_MOVE_RIGHT: u8 = 0x04;
const LCD_MOVE_LEFT: u8 = 0x00;

// flags for function set
const LCD_8BITMODE: u8 = 0x10;
#[allow(dead_code)]
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
#[allow(dead_code)]
const LCD_1LINE: u8 = 0x00;
#[allow(dead_code)]
const LCD_5X10_DOTS: u8 = 0x04;
const LCD_5X8_DOTS: u8 = 0x00;

// shared  function flags
const LCD_DISPLAY_ON: u8 = 0x04;

impl RgbLCD {
    /// Create a and initialize a LCD backlight structure
    ///
    pub fn new() -> Self {
        RgbLCD {
            display_function: LCD_2LINE | LCD_5X8_DOTS,
            display_control: LCD_DISPLAY_ON,
            display_mode: LCD_8BITMODE,
        }
    }

    /// Initialize the LCD display
    ///
    /// Args:
    /// * i2c - An initialized I2C device
    ///
    /// Returns
    /// * empty or I2C write error
    pub fn init<E, I2C: Write<Error = E>, D: DelayUs<u32>>(
        &mut self,
        i2c: &mut I2C,
        block: D,
    ) -> Result<(), E> {
        let mut block = block;
        // SEE PAGE 45/46 FOR INITIALIZATION SPECIFICATION!
        // according to data sheet, we need at least 40ms after power rises above 2.7V
        // before sending commands. So we'll wait 50
        // this is according to the hitachi HD44780 data sheet
        // page 45 figure 23
        block.delay_us(50000);

        // Send function set command sequence
        send_command(i2c, LCD_FUNCTION_SET | self.display_function)?;
        block.delay_us(4500); // wait more than 4.1ms

        // second try
        send_command(i2c, LCD_FUNCTION_SET | self.display_function)?;
        block.delay_us(150);

        // third go
        send_command(i2c, LCD_FUNCTION_SET | self.display_function)?;

        // finally, set # lines, font size, etc.
        send_command(i2c, LCD_FUNCTION_SET | self.display_function)?;

        self.switch_display_on(i2c)?;
        self.clear_display(i2c, block)?;

        // Initialize to default text direction (for romance languages)
        self.display_mode |= LCD_ENTRY_LEFT | LCD_ENTRY_SHIFT_DECREMENT;
        send_command(i2c, LCD_ENTRY_MODESET | self.display_mode)?;

        // backlight init
        const REG_MODE1: u8 = 0x00;
        const REG_MODE2: u8 = 0x01;
        const REG_OUTPUT: u8 = 0x08;

        set_register(i2c, REG_MODE1, 0)?;
        // set LEDs controllable by both PWM and GRPPWM registers
        set_register(i2c, REG_OUTPUT, 0xFF)?;
        // set MODE2 values
        // 0010 0000 -> 0x20  (DMBLNK to 1, ie blinky mode)
        set_register(i2c, REG_MODE2, 0x20)?;

        self.set_color(i2c, Color::Green)
    }

    /// clear display, set cursor position to zero
    pub fn clear_display<E, I2C: Write<Error = E>, D: DelayUs<u32>>(
        &self,
        i2c: &mut I2C,
        block: D,
    ) -> Result<(), E> {
        const LCD_CLEAR_DISPLAY: u8 = 0x01;
        send_command(i2c, LCD_CLEAR_DISPLAY)?;
        let mut block = block;
        block.delay_us(2000); // this command takes a long time!
        Ok(())
    }

    /// set cursor position to zero
    pub fn home<E, I2C: Write<Error = E>, D: DelayUs<u32>>(
        &self,
        i2c: &mut I2C,
        block: D,
    ) -> Result<(), E> {
        const LCD_RETURN_HOME: u8 = 0x02;
        send_command(i2c, LCD_RETURN_HOME)?;
        let mut block = block;
        block.delay_us(2000); // this command takes a long time!
        Ok(())
    }

    pub fn switch_display_off<E, I2C: Write<Error = E>>(&mut self, i2c: &mut I2C) -> Result<(), E> {
        const LCD_DISPLAY_ON: u8 = 0x04;
        self.display_control &= !LCD_DISPLAY_ON;
        send_command(i2c, LCD_DISPLAY_CONTROL | self.display_control)
    }

    pub fn switch_display_on<E, I2C: Write<Error = E>>(&mut self, i2c: &mut I2C) -> Result<(), E> {
        const LCD_DISPLAY_ON: u8 = 0x04;
        self.display_control |= LCD_DISPLAY_ON;
        send_command(i2c, LCD_DISPLAY_CONTROL | self.display_control)
    }

    pub fn switch_cursor_blinking_off<E, I2C: Write<Error = E>>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        const LCD_BLINK_ON: u8 = 0x01;
        self.display_control &= !LCD_BLINK_ON;
        send_command(i2c, LCD_DISPLAY_CONTROL | self.display_control)
    }

    pub fn switch_cursor_blinking_on<E, I2C: Write<Error = E>>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        const LCD_BLINK_ON: u8 = 0x01;
        self.display_control |= LCD_BLINK_ON;
        send_command(i2c, LCD_DISPLAY_CONTROL | self.display_control)
    }

    pub fn hide_cursor<E, I2C: Write<Error = E>>(&mut self, i2c: &mut I2C) -> Result<(), E> {
        const LCD_CURSOR_ON: u8 = 0x02;
        self.display_control &= !LCD_CURSOR_ON;
        send_command(i2c, LCD_DISPLAY_CONTROL | self.display_control)
    }

    pub fn show_cursor<E, I2C: Write<Error = E>>(&mut self, i2c: &mut I2C) -> Result<(), E> {
        const LCD_CURSOR_ON: u8 = 0x02;
        self.display_control |= LCD_CURSOR_ON;
        send_command(i2c, LCD_DISPLAY_CONTROL | self.display_control)
    }

    pub fn scroll_display_left<E, I2C: Write<Error = E>>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        // This commands scroll the display without changing the RAM
        send_command(i2c, LCD_CURSOR_SHIFT | LCD_DISPLAY_MOVE | LCD_MOVE_LEFT)
    }

    pub fn scroll_display_right<E, I2C: Write<Error = E>>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        send_command(i2c, LCD_CURSOR_SHIFT | LCD_DISPLAY_MOVE | LCD_MOVE_RIGHT)
    }

    /// Text that flows Left to Right
    pub fn set_left_to_right_text_flow<E, I2C: Write<Error = E>>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        self.display_mode |= LCD_ENTRY_LEFT;
        send_command(i2c, LCD_ENTRY_MODESET | self.display_mode)
    }

    /// Text that flows Right to Left
    pub fn set_right_to_left_text_flow<E, I2C: Write<Error = E>>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        self.display_mode &= !LCD_ENTRY_RIGHT;
        send_command(i2c, LCD_ENTRY_MODESET | self.display_mode)
    }

    /// 'right justify' text from the cursor
    pub fn switch_autoscrolling_on<E, I2C: Write<Error = E>>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        self.display_mode |= LCD_ENTRY_SHIFT_INCREMENT;
        send_command(i2c, LCD_ENTRY_MODESET | self.display_mode)
    }

    /// 'left justify' text from the cursor
    pub fn switch_autoscrolling_off<E, I2C: Write<Error = E>>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        self.display_mode &= !LCD_ENTRY_SHIFT_INCREMENT;
        send_command(i2c, LCD_ENTRY_MODESET | self.display_mode)
    }

    /// Allows us to fill the first 8 CGRAM locations with custom characters
    /// location is in range 0..7
    pub fn create_custom_characters<E, I2C: Write<Error = E>>(
        &self,
        i2c: &mut I2C,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), E> {
        if location <= 8 {
            panic!("Location must be in range 0..7");
        }
        send_command(i2c, LCD_SET_CGRAM_ADDR | (location << 3))?;
        let data: [u8; 9] = [
            0x40, charmap[0], charmap[1], charmap[2], charmap[3], charmap[4], charmap[5],
            charmap[6], charmap[7],
        ];
        i2c.write(LCD_ADDRESS, &data)?;
        Ok(())
    }

    /// Position the cursor
    pub fn set_cursor<E, I2C: Write<Error = E>>(
        &self,
        i2c: &mut I2C,
        col: u8,
        row: u8,
    ) -> Result<(), E> {
        let pos: u8 = 0x80 + col + row * (0xc0 - 0x80);
        let data: [u8; 2] = [0x80, pos];
        i2c.write(LCD_ADDRESS, &data)?;
        Ok(())
    }

    /// Send a byte
    pub fn write_byte<E, I2C: Write<Error = E>>(&self, i2c: &mut I2C, value: u8) -> Result<(), E> {
        let data: [u8; 2] = [0x40, value];
        i2c.write(LCD_ADDRESS, &data)?;
        Ok(())
    }

    /// Send text

    pub fn switch_blink_backlight_on<E, I2C: Write<Error = E>>(
        &self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        set_register(i2c, 0x07, 0x17)?; // blink every second
        set_register(i2c, 0x06, 0x7f) // half on, half off
    }

    pub fn switch_blink_backlight_off<E, I2C: Write<Error = E>>(
        &self,
        i2c: &mut I2C,
    ) -> Result<(), E> {
        set_register(i2c, 0x07, 0x00)?;
        set_register(i2c, 0x06, 0xff)
    }

    /// Set the backlight color
    pub fn set_color<E, I2C: Write<Error = E>>(
        &self,
        i2c: &mut I2C,
        color: Color,
    ) -> Result<(), E> {
        let (red, green, blue) = match color {
            Color::White => (255, 255, 255),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::RGB(red, green, blue) => (red, green, blue),
        };
        const REG_RED: u8 = 0x04; // pwm2
        const REG_GREEN: u8 = 0x03; // pwm1
        const REG_BLUE: u8 = 0x02; // pwm0
        set_register(i2c, REG_RED, red)?;
        set_register(i2c, REG_GREEN, green)?;
        set_register(i2c, REG_BLUE, blue)?;
        Ok(())
    }
}

fn set_register<E, I2C: Write<Error = E>>(i2c: &mut I2C, address: u8, value: u8) -> Result<(), E> {
    let data: [u8; 2] = [address, value];
    i2c.write(RGB_ADDRESS, &data)?; // blocking transmission
    Ok(())
}

fn send_command<E, I2C: Write<Error = E>>(i2c: &mut I2C, command: u8) -> Result<(), E> {
    let data: [u8; 2] = [0x80, command];
    i2c.write(LCD_ADDRESS, &data)?;
    Ok(())
}
