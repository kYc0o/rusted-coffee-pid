//! Button input handler
//!
//! Detects button presses on GPIO9 with debouncing
//! Sends events to a channel for the brew controller

use ariel_os::gpio::{Input, Pull};
use ariel_os::hal::peripherals::GPIO9;
use ariel_os::time::Timer;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[derive(Debug, Clone, Copy)]
pub enum ButtonEvent {
    Pressed,
}

/// Button task that monitors GPIO9 for presses
/// Detects falling edge (high -> low, active-low with pull-up)
/// Hardware debouncing handles signal stability
pub async fn button_task(
    button_pin: GPIO9,
    event_channel: &'static Channel<CriticalSectionRawMutex, ButtonEvent, 1>,
) {
    let button = Input::builder(button_pin, Pull::Up).build();
    
    let mut last_state = button.is_high();
    
    loop {
        let current_state = button.is_high();
        
        // Detect falling edge (high -> low, button pressed with active-low logic)
        if last_state && !current_state {
            // Send press event immediately (hardware debounce handles signal stability)
            let _ = event_channel.try_send(ButtonEvent::Pressed);
            
            // Wait for button release (low -> high)
            while !button.is_high() {
                Timer::after_millis(10).await;
            }
            
            // Brief delay to avoid re-triggering on release
            Timer::after_millis(10).await;
            last_state = true;
        } else {
            last_state = current_state;
        }
        
        // Poll button every 10ms
        Timer::after_millis(10).await;
    }
}
