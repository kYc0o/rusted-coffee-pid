//! SSD1306 Command Definitions
//!
//! All control commands for the SSD1306 OLED controller

/// Fundamental Commands
#[allow(dead_code)]
pub mod fundamental {
    pub const SET_CONTRAST: u8 = 0x81;
    pub const DISPLAY_ALL_ON_RESUME: u8 = 0xA4;
    pub const DISPLAY_ALL_ON: u8 = 0xA5;
    pub const NORMAL_DISPLAY: u8 = 0xA6;
    pub const INVERT_DISPLAY: u8 = 0xA7;
}

/// Scrolling Commands
#[allow(dead_code)]
pub mod scrolling {
    pub const RIGHT_HORIZONTAL_SCROLL: u8 = 0x26;
    pub const LEFT_HORIZONTAL_SCROLL: u8 = 0x27;
    pub const VERTICAL_RIGHT_HORIZONTAL_SCROLL: u8 = 0x29;
    pub const VERTICAL_LEFT_HORIZONTAL_SCROLL: u8 = 0x2A;
    pub const DEACTIVATE_SCROLL: u8 = 0x2E;
    pub const ACTIVATE_SCROLL: u8 = 0x2F;
    pub const SET_VERTICAL_SCROLL_AREA: u8 = 0xA3;
}

/// Addressing Commands
#[allow(dead_code)]
pub mod addressing {
    pub const SET_COLUMN_ADDRESS: u8 = 0x21;
    pub const SET_PAGE_ADDRESS: u8 = 0x22;
    pub const SET_ADDRESSING_MODE: u8 = 0x20;
    
    // Addressing modes
    pub const ADDRESSING_HORIZONTAL: u8 = 0x00;
    pub const ADDRESSING_VERTICAL: u8 = 0x01;
    pub const ADDRESSING_PAGE: u8 = 0x02;
}

/// Hardware Configuration Commands
pub mod hardware {
    pub const SET_DISPLAY_START_LINE: u8 = 0x40;
    pub const SET_SEGMENT_REMAP: u8 = 0xA0;
    pub const SET_MULTIPLEX_RATIO: u8 = 0xA8;
    pub const COM_OUTPUT_DIRECTION: u8 = 0xC0;
    pub const SET_DISPLAY_OFFSET: u8 = 0xD3;
    pub const SET_COM_PINS: u8 = 0xDA;
}

/// Timing and Driving Scheme Commands
pub mod timing {
    pub const SET_DISPLAY_CLOCK_DIVIDE: u8 = 0xD5;
    pub const SET_PRECHARGE_PERIOD: u8 = 0xD9;
    pub const SET_VCOMH_DESELECT_LEVEL: u8 = 0xDB;
}

/// Charge Pump Commands
pub mod charge_pump {
    pub const SET_CHARGE_PUMP: u8 = 0x8D;
}

/// Display Control Commands
pub mod display {
    pub const DISPLAY_OFF: u8 = 0xAE;
    pub const DISPLAY_ON: u8 = 0xAF;
}

/// Initialization sequence for 128x64 display
pub fn init_sequence_128x64() -> [u8; 25] {
    [
        display::DISPLAY_OFF,                    // 0xAE - Display OFF
        timing::SET_DISPLAY_CLOCK_DIVIDE,        // 0xD5
        0x80,                                     // Default clock divide ratio
        hardware::SET_MULTIPLEX_RATIO,           // 0xA8
        0x3F,                                     // 63 (for 64 pixel height)
        hardware::SET_DISPLAY_OFFSET,            // 0xD3
        0x00,                                     // No offset
        hardware::SET_DISPLAY_START_LINE | 0x00, // 0x40 - Start line 0
        charge_pump::SET_CHARGE_PUMP,            // 0x8D
        0x14,                                     // Enable charge pump
        addressing::SET_ADDRESSING_MODE,         // 0x20
        addressing::ADDRESSING_HORIZONTAL,       // Horizontal addressing
        hardware::SET_SEGMENT_REMAP | 0x01,      // 0xA1 - Remap segments
        hardware::COM_OUTPUT_DIRECTION | 0x08,   // 0xC8 - Remap COM output
        hardware::SET_COM_PINS,                  // 0xDA
        0x12,                                     // COM pins configuration
        fundamental::SET_CONTRAST,               // 0x81
        0xCF,                                     // Contrast value
        timing::SET_PRECHARGE_PERIOD,            // 0xD9
        0xF1,                                     // Precharge period
        timing::SET_VCOMH_DESELECT_LEVEL,        // 0xDB
        0x40,                                     // VCOMH deselect level
        fundamental::NORMAL_DISPLAY,             // 0xA6 - Normal display
        fundamental::DISPLAY_ALL_ON_RESUME,      // 0xA4 - Resume to RAM
        display::DISPLAY_ON,                     // 0xAF - Display ON
    ]
}
