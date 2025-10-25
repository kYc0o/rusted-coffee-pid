//! SSD1306 Display Driver with embedded-graphics support
//!
//! Async-compatible wrapper around SSD1306 using embedded-graphics for drawing
//! and async I2C for communication

use embedded_hal_async::i2c::I2c;
use embedded_graphics::{
    prelude::*,
    pixelcolor::BinaryColor,
};
use super::commands;

/// Display dimensions
pub const DISPLAY_WIDTH: usize = 128;
pub const DISPLAY_HEIGHT: usize = 64;
pub const DISPLAY_BUFFER_SIZE: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT) / 8; // 1024 bytes

/// I2C Address for SSD1306
const I2C_ADDRESS: u8 = 0x3C;

/// Control byte for sending commands
const CONTROL_BYTE_CMD: u8 = 0x00;
/// Control byte for sending data
const CONTROL_BYTE_DATA: u8 = 0x40;

/// Display errors
#[derive(Debug, Clone, Copy)]
pub enum Error {
    I2cError,
    InvalidCoordinates,
    DrawError,
}

impl From<core::convert::Infallible> for Error {
    fn from(_: core::convert::Infallible) -> Self {
        Error::DrawError
    }
}

/// Display mode
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum DisplayMode {
    Normal,
    Inverted,
}

/// SSD1306 Display with manual framebuffer for embedded-graphics compatibility
pub struct Display<I2C: I2c> {
    i2c: I2C,
    /// Framebuffer: 128x64 pixels = 1024 bytes (8 pixels per byte, LSB = top pixel)
    buffer: [u8; DISPLAY_BUFFER_SIZE],
    initialized: bool,
}

impl<I2C: I2c> Display<I2C> {
    /// Create a new display instance
    pub fn new(i2c: I2C) -> Self {
        Display {
            i2c,
            buffer: [0; DISPLAY_BUFFER_SIZE],
            initialized: false,
        }
    }

    /// Initialize the display
    pub async fn init(&mut self) -> Result<(), Error> {
        // Send initialization sequence
        let init_seq = commands::init_sequence_128x64();
        self.send_commands(&init_seq).await?;
        self.initialized = true;
        Ok(())
    }

    /// Clear the display buffer
    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    /// Set a pixel at (x, y) to on or off
    pub fn set_pixel(&mut self, x: usize, y: usize, on: bool) -> Result<(), Error> {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return Err(Error::InvalidCoordinates);
        }

        let byte_index = (y / 8) * DISPLAY_WIDTH + x;
        let bit = (y % 8) as u8;

        if on {
            self.buffer[byte_index] |= 1 << bit;
        } else {
            self.buffer[byte_index] &= !(1 << bit);
        }

        Ok(())
    }

    /// Get the underlying buffer as a mutable slice for direct access
    #[allow(dead_code)]
    pub fn buffer_mut(&mut self) -> &mut [u8; DISPLAY_BUFFER_SIZE] {
        &mut self.buffer
    }

    /// Flush the framebuffer to the display via I2C
    pub async fn flush(&mut self) -> Result<(), Error> {
        if !self.initialized {
            return Err(Error::I2cError);
        }

        // Set column address (0-127)
        self.send_commands(&[
            commands::addressing::SET_COLUMN_ADDRESS,
            0,
            127,
        ]).await?;

        // Set page address (0-7)
        self.send_commands(&[
            commands::addressing::SET_PAGE_ADDRESS,
            0,
            7,
        ]).await?;

        // Copy buffer to avoid borrow checker issues
        let buffer_copy = self.buffer;
        
        // Send all framebuffer data at once
        self.send_data(&buffer_copy).await?;

        Ok(())
    }

    /// Turn display on
    #[allow(dead_code)]
    pub async fn display_on(&mut self) -> Result<(), Error> {
        self.send_commands(&[commands::display::DISPLAY_ON]).await
    }

    /// Turn display off
    #[allow(dead_code)]
    pub async fn display_off(&mut self) -> Result<(), Error> {
        self.send_commands(&[commands::display::DISPLAY_OFF]).await
    }

    /// Set display contrast (0-255)
    #[allow(dead_code)]
    pub async fn set_contrast(&mut self, value: u8) -> Result<(), Error> {
        self.send_commands(&[commands::fundamental::SET_CONTRAST, value]).await
    }

    /// Send command bytes to the display
    async fn send_commands(&mut self, commands: &[u8]) -> Result<(), Error> {
        let mut tx_buffer = [CONTROL_BYTE_CMD; 33]; // 1 control byte + 32 command bytes max
        
        if commands.len() > 32 {
            return Err(Error::I2cError);
        }

        // Copy commands after control byte
        tx_buffer[1..commands.len() + 1].copy_from_slice(commands);

        self.i2c
            .write(I2C_ADDRESS, &tx_buffer[..commands.len() + 1])
            .await
            .map_err(|_| Error::I2cError)
    }

    /// Send data bytes to the display
    async fn send_data(&mut self, data: &[u8]) -> Result<(), Error> {
        // I2C has a limit on how much we can send at once
        const CHUNK_SIZE: usize = 32;
        let mut tx_buffer = [CONTROL_BYTE_DATA; CHUNK_SIZE + 1];

        for chunk in data.chunks(CHUNK_SIZE) {
            // Copy data after control byte
            tx_buffer[1..chunk.len() + 1].copy_from_slice(chunk);

            self.i2c
                .write(I2C_ADDRESS, &tx_buffer[..chunk.len() + 1])
                .await
                .map_err(|_| Error::I2cError)?;
        }

        Ok(())
    }
}

// Implement DrawTarget for embedded-graphics compatibility
impl<I2C: I2c> DrawTarget for Display<I2C> {
    type Color = BinaryColor;
    type Error = Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Check bounds
            if coord.x < 0 || coord.x >= DISPLAY_WIDTH as i32 || 
               coord.y < 0 || coord.y >= DISPLAY_HEIGHT as i32 {
                continue;
            }
            
            let x = coord.x as usize;
            let y = coord.y as usize;
            self.set_pixel(x, y, color == BinaryColor::On).ok();
        }
        Ok(())
    }
}

impl<I2C: I2c> OriginDimensions for Display<I2C> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}
