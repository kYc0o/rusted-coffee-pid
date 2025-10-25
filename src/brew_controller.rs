//! Brew controller state machine
//! 
//! Manages the two display states:
//! - IDLE: Shows large temperature display
//! - BREWING: Shows large timer with small temperature below, stops at 60s

use ariel_os::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrewState {
    /// Idle state: displays temperature in large font
    Idle,
    /// Brewing state: timer running, shows timer in large font and temp in small
    Brewing {
        start_time: Instant,
    },
    /// Complete state: timer finished at 60s, still showing brew info
    Complete {
        start_time: Instant,
    },
}

pub struct BrewController {
    pub state: BrewState,
}

impl BrewController {
    pub fn new() -> Self {
        BrewController {
            state: BrewState::Idle,
        }
    }

    /// Start brewing
    pub fn start_brew(&mut self) {
        if self.state == BrewState::Idle {
            self.state = BrewState::Brewing {
                start_time: Instant::now(),
            };
        }
    }

    /// Update state - check if brew is complete
    pub fn update(&mut self) {
        if let BrewState::Brewing { start_time } = self.state {
            let elapsed_ms = start_time.elapsed().as_millis() as u64;
            if elapsed_ms >= 60_000 {
                // Transition to complete state
                self.state = BrewState::Complete { start_time };
            }
        }
    }

    /// Reset to idle state
    /// Used for resetting after a brew cycle completes (future feature)
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.state = BrewState::Idle;
    }

    /// Get elapsed time in tenths of seconds (0-600)
    pub fn elapsed_tenths(&self) -> u32 {
        match self.state {
            BrewState::Idle => 0,
            BrewState::Brewing { start_time } => {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                (elapsed_ms / 100).min(600) as u32
            }
            BrewState::Complete { start_time } => {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                (elapsed_ms / 100).min(600) as u32
            }
        }
    }
}
