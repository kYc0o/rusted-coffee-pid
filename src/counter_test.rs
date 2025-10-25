//! Display test showing temperature and timer on SSD1306
//!
//! State machine:
//! - IDLE: Large font temperature display
//! - BREWING: Large font timer, small font temperature below
//! - COMPLETE: Same as brewing, but timer stopped at 60s

use ariel_os::{
    debug::log::info,
    i2c::controller::{Kilohertz, highest_freq_in},
    time::{Timer, Instant},
};

use embedded_graphics::{
    mono_font::{ascii::FONT_9X15_BOLD, ascii::FONT_5X8, MonoTextStyle},
    text::{Text, Alignment},
    prelude::*,
    pixelcolor::BinaryColor,
};

use crate::pins;
use crate::drivers::ssd1306_async::Display;
use crate::comm::TEMP_CHANNEL;
use crate::brew_controller::{BrewController, BrewState};
use crate::button::ButtonEvent;

/// Ultra-fast temperature formatter without using core::fmt
/// Formats as "Temp: XX.X°C" with proper spacing
#[inline]
fn format_temp_fast(temp: f32) -> &'static str {
    // Static buffer for formatted string (reused across calls)
    static mut TEMP_BUF: [u8; 16] = [b' '; 16];
    
    unsafe {
        let mut pos = 0;
        
        // "Temp: "
        TEMP_BUF[pos..pos+6].copy_from_slice(b"Temp: ");
        pos += 6;
        
        // Handle negative temperatures
        let abs_temp = if temp < 0.0 {
            TEMP_BUF[pos] = b'-';
            pos += 1;
            -temp
        } else {
            temp
        };
        
        let int_part = abs_temp as i32;
        
        // Integer part - handle up to 3 digits (sufficient for -50 to 150°C)
        if int_part >= 100 {
            TEMP_BUF[pos] = b'0' + ((int_part / 100) as u8);
            pos += 1;
        }
        
        if int_part >= 10 {
            TEMP_BUF[pos] = b'0' + (((int_part / 10) % 10) as u8);
            pos += 1;
        } else if int_part >= 100 {
            // Add leading zero if we had hundreds digit
            TEMP_BUF[pos] = b'0';
            pos += 1;
        }
        
        TEMP_BUF[pos] = b'0' + ((int_part % 10) as u8);
        pos += 1;
        
        // Decimal point
        TEMP_BUF[pos] = b'.';
        pos += 1;
        
        // Fractional part (1 decimal place)
        let frac = ((abs_temp - (int_part as f32)) * 10.0) as u8;
        TEMP_BUF[pos] = b'0' + (frac % 10);
        pos += 1;
        
        // "°C" - UTF-8 encoding for °
        TEMP_BUF[pos] = 0xC2; // First byte of ° in UTF-8
        pos += 1;
        TEMP_BUF[pos] = 0xB0; // Second byte of ° in UTF-8
        pos += 1;
        TEMP_BUF[pos] = b'C';
        pos += 1;
        
        // Convert to str (safe because we control the content)
        core::str::from_utf8_unchecked(&TEMP_BUF[..pos])
    }
}

/// Ultra-fast timer formatter
/// Formats as "MM.T" (e.g., "12.5" for 12.5 seconds)
#[inline]
fn format_timer_fast(tenths: u32) -> &'static str {
    static mut TIMER_BUF: [u8; 6] = [b' '; 6];
    
    unsafe {
        let seconds = tenths / 10;
        let frac = tenths % 10;
        
        let mut pos = 0;
        
        // Tens digit
        if seconds >= 10 {
            TIMER_BUF[pos] = b'0' + ((seconds / 10) as u8);
            pos += 1;
        } else {
            // Leading space for alignment
            TIMER_BUF[pos] = b' ';
            pos += 1;
        }
        
        // Ones digit
        TIMER_BUF[pos] = b'0' + ((seconds % 10) as u8);
        pos += 1;
        
        // Decimal point
        TIMER_BUF[pos] = b'.';
        pos += 1;
        
        // Tenths digit
        TIMER_BUF[pos] = b'0' + (frac as u8);
        pos += 1;
        
        core::str::from_utf8_unchecked(&TIMER_BUF[..pos])
    }
}

pub async fn counter_test(i2c_sda: ariel_os::hal::peripherals::GPIO2, i2c_scl: ariel_os::hal::peripherals::GPIO0) {
    info!("Starting display counter test with temperature");

    // Initialize I2C with 100kHz frequency
    let mut i2c_config = ariel_os::hal::i2c::controller::Config::default();
    i2c_config.frequency = const { highest_freq_in(Kilohertz::kHz(100)..=Kilohertz::kHz(400)) };
    
    let i2c_bus = pins::SensorI2c::new(i2c_sda, i2c_scl, i2c_config);

    // Initialize display
    let mut display = Display::new(i2c_bus);
    
    if let Err(_) = display.init().await {
        info!("Failed to initialize display");
        return;
    }

    info!("Display initialized successfully");

    // Define text styles
    let title_style = MonoTextStyle::new(&FONT_9X15_BOLD, BinaryColor::On);
    let temp_style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);

    // Track last known temperature
    let mut last_temp: Option<f32> = None;
    
    // Create brew controller state machine
    let mut brew = BrewController::new();
    
    // Main display loop - runs indefinitely
    let mut last_update = Instant::now();
    const UPDATE_INTERVAL_MS: u64 = 100;  // 10Hz update

    loop {
        // Try to read latest temperature (non-blocking)
        while let Ok(temp_option) = TEMP_CHANNEL.try_receive() {
            if let Some(t) = temp_option {
                last_temp = Some(t);
            }
        }
        
        // Check for button events
        while let Ok(ButtonEvent::Pressed) = crate::comm::BREW_BUTTON_CHANNEL.try_receive() {
            brew.start_brew();
            info!("Brew started!");
        }
        
        // Update brew state (check if complete)
        brew.update();
        
        // Calculate time until next update
        let now = Instant::now();
        let since_last_update = now.duration_since(last_update).as_millis() as u64;
        
        if since_last_update >= UPDATE_INTERVAL_MS {
            // Clear display buffer
            display.clear();
            
            match brew.state {
                BrewState::Idle => {
                    // IDLE: Show large temperature
                    let temp_str = if let Some(t) = last_temp {
                        format_temp_fast(t)
                    } else {
                        "Temp:  --.-°C"
                    };
                    
                    // Draw "Ready" label
                    let _ = Text::with_alignment(
                        "Ready",
                        Point::new(64, 10),
                        temp_style,
                        Alignment::Center,
                    ).draw(&mut display);
                    
                    // Draw large temperature (centered, large font)
                    let _ = Text::with_alignment(
                        temp_str,
                        Point::new(64, 40),
                        title_style,
                        Alignment::Center,
                    ).draw(&mut display);
                }
                BrewState::Brewing { .. } | BrewState::Complete { .. } => {
                    // BREWING/COMPLETE: Show timer and temperature
                    let counter_tenths = brew.elapsed_tenths();
                    let timer_str = format_timer_fast(counter_tenths);
                    
                    let temp_str = if let Some(t) = last_temp {
                        format_temp_fast(t)
                    } else {
                        "Temp:  --.-°C"
                    };
                    
                    // Draw "Brewing" label (or "Complete" if done)
                    let label = match brew.state {
                        BrewState::Complete { .. } => "Complete!",
                        _ => "Brewing...",
                    };
                    
                    let _ = Text::with_alignment(
                        label,
                        Point::new(64, 8),
                        temp_style,
                        Alignment::Center,
                    ).draw(&mut display);
                    
                    // Draw timer (large font, centered)
                    let _ = Text::with_alignment(
                        timer_str,
                        Point::new(64, 35),
                        title_style,
                        Alignment::Center,
                    ).draw(&mut display);
                    
                    // Draw temperature (small font, bottom left)
                    let _ = Text::new(temp_str, Point::new(0, 55), temp_style)
                        .draw(&mut display);
                }
            }
            
            // Flush to display (async)
            if let Err(_) = display.flush().await {
                info!("Display flush failed");
            }
            
            last_update = now;
        }
        
        // Sleep until next potential update (yield to other tasks)
        let sleep_time = UPDATE_INTERVAL_MS.saturating_sub(since_last_update);
        Timer::after_millis(sleep_time).await;
    }
}
