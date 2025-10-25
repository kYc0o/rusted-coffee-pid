//! Async SSD1306 OLED Display Driver
//!
//! A minimal, async-compatible driver for SSD1306 128x64 OLED displays
//! using I2C communication with embedded-graphics for drawing primitives.

pub mod commands;
pub mod display;

pub use display::Display;
