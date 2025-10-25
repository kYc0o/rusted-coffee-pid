//! Temperature sensor task for TSIC 306 with Ariel OS
//!
//! This module provides an async task that reads the TSIC 306 sensor
//! and shares temperature data with display code via embassy_sync channels.

use ariel_os::gpio::{Input, Pull};
use ariel_os::hal::peripherals::GPIO1;
use ariel_os::time::Timer;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::drivers::tsic306_async::TsicSensor;

/// Async task that continuously reads TSIC 306 temperature sensor
/// 
/// Reads from the sensor every 100ms and sends temperature to the provided channel
/// 
/// # Arguments
/// - `gpio_pin`: GPIO1 peripheral for TSIC 306 ZACwire signal
/// - `channel`: Static reference to temperature channel for display
pub async fn sensor_task(gpio_pin: GPIO1, channel: &'static Channel<CriticalSectionRawMutex, Option<f32>, 1>) {
    // Convert GPIO peripheral to Input with default pull-up
    let pin = Input::builder(gpio_pin, Pull::Up).build();
    let mut sensor = TsicSensor::new(pin);
    
    // Give sensor time to stabilize on boot
    Timer::after_millis(500).await;
    
    let mut consecutive_errors = 0;
    
    loop {
        // Attempt to read temperature in a blocking manner
        // Since this runs in an executor context, we use busy-wait which is fine for sensor reads
        match sensor.read_zacwire_blocking() {
            Ok(temp) => {
                // Send temperature to the channel (non-blocking try_send)
                let _ = channel.try_send(Some(temp));
                consecutive_errors = 0; // Reset error counter on success
            }
            Err(_e) => {
                consecutive_errors += 1;
                // Only log after several consecutive errors
                if consecutive_errors > 10 {
                    ariel_os::debug::log::warn!("TSIC: Multiple read failures");
                }
                // Signal error through channel only on persistent failures
                if consecutive_errors > 5 {
                    let _ = channel.try_send(None);
                }
            }
        }
        
        // Wait 100ms before next read (TSIC samples at ~10Hz)
        Timer::after_millis(100).await;
    }
}
