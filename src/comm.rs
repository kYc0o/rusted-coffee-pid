//! Communication channels for cross-task data sharing

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use crate::button::ButtonEvent;

/// Temperature channel: sensor task sends Option<f32> (None on error)
pub static TEMP_CHANNEL: Channel<CriticalSectionRawMutex, Option<f32>, 1> = Channel::new();

/// Button event channel: button task sends ButtonEvent (e.g., Pressed)
pub static BREW_BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 1> = Channel::new();
