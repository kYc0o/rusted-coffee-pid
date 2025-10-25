//! TSIC 306 temperature sensor driver using ZACwire protocol
//!
//! Reads temperature from TSIC 306 sensor via single GPIO input pin using ZACwire protocol.

use ariel_os::gpio::Input;

/// TSIC 306 temperature sensor reader using ZACwire protocol
pub struct TsicSensor {
    pin: Input,
}

impl TsicSensor {
    /// Create new TSIC sensor from GPIO input
    pub fn new(pin: Input) -> Self {
        Self { pin }
    }

    /// Read temperature synchronously (blocking)
    /// This is designed to run in its own thread
    pub fn read_zacwire_blocking(&mut self) -> Result<f32, TsicError> {
        // Wait for falling edge (start bit)
        while self.pin.is_high() {}
        
        // Measure Tstrobe timing
        let start = ariel_os::time::Instant::now();
        while self.pin.is_low() {}
        let tstrobe = start.elapsed();
        
        // Validate timing (should be 40-125µs)
        if tstrobe.as_micros() < 40 || tstrobe.as_micros() > 125 {
            return Err(TsicError::InvalidStrobe);
        }
        
        let bit_period_us = (tstrobe.as_micros() as u32) * 2;
        
        // Read first byte
        let byte1 = self.read_byte_blocking(bit_period_us)?;
        
        // Wait for inter-byte gap
        while self.pin.is_high() {}
        while self.pin.is_low() {}
        
        // Read second byte
        let byte2 = self.read_byte_blocking(bit_period_us)?;
        
        // Convert to temperature: (raw * 200 / 2047) - 50
        let raw = ((byte1 as u16) << 8) | (byte2 as u16);
        let temperature = (raw as f32 * 200.0 / 2047.0) - 50.0;
        
        Ok(temperature)
    }

    fn read_byte_blocking(&mut self, bit_period_us: u32) -> Result<u8, TsicError> {
        let mut data = 0u8;
        let mut parity = 0u8;
        
        // Read 8 data bits + 1 parity bit
        for bit_pos in 0..9 {
            // Wait for falling edge
            while self.pin.is_high() {}
            
            // Sample at bit midpoint using busy-wait (blocking context)
            let start = ariel_os::time::Instant::now();
            while start.elapsed().as_micros() < (bit_period_us / 2) as u64 {}
            
            if self.pin.is_high() {
                if bit_pos < 8 {
                    data |= 1 << (7 - bit_pos);
                }
                parity ^= 1;
            }
            
            // Wait for bit to complete (rising edge)
            while self.pin.is_low() {}
        }
        
        // Check even parity
        if parity != 0 {
            return Err(TsicError::ParityError);
        }
        
        Ok(data)
    }
}

/// Errors that can occur during TSIC sensor reads
#[derive(Debug)]
pub enum TsicError {
    /// Strobe signal timing was invalid
    InvalidStrobe,
    /// Parity check failed
    ParityError,
}
