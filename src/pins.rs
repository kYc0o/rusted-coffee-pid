use ariel_os::hal::{i2c, peripherals};

#[cfg(context = "espressif-esp32-s3-devkitc-1")]
pub type SensorI2c = i2c::controller::I2C0;
#[cfg(context = "espressif-esp32-s3-devkitc-1")]
ariel_os::hal::define_peripherals!(Peripherals {
    i2c_sda: GPIO2,
    i2c_scl: GPIO0,
    tsic_zacwire: GPIO1,
    brew_button: GPIO9,
});
