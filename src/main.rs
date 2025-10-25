#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]

mod pins;
mod counter_test;
mod temperature_sensor;
mod drivers;
mod comm;
mod button;
mod brew_controller;

use ariel_os::asynch::Spawner;
use crate::comm::{TEMP_CHANNEL, BREW_BUTTON_CHANNEL};

// Display task
#[ariel_os::task]
async fn display_task(i2c_sda: ariel_os::hal::peripherals::GPIO2, i2c_scl: ariel_os::hal::peripherals::GPIO0) {
    counter_test::counter_test(i2c_sda, i2c_scl).await;
}

// Temperature sensor task
#[ariel_os::task]
async fn sensor_task(pin: ariel_os::hal::peripherals::GPIO1) {
    temperature_sensor::sensor_task(pin, &TEMP_CHANNEL).await;
}

// Button input task
#[ariel_os::task]
async fn button_task_spawn(pin: ariel_os::hal::peripherals::GPIO9) {
    button::button_task(pin, &BREW_BUTTON_CHANNEL).await;
}

#[ariel_os::spawner(autostart, peripherals)]
fn main(spawner: Spawner, peripherals: pins::Peripherals) {
    // Extract pins needed by tasks
    let tsic_pin = peripherals.tsic_zacwire;
    let i2c_sda = peripherals.i2c_sda;
    let i2c_scl = peripherals.i2c_scl;
    let brew_button = peripherals.brew_button;
    
    // Spawn display task with I2C pins
    spawner.spawn(display_task(i2c_sda, i2c_scl)).unwrap();

    // Spawn temperature sensor task with TSIC pin
    spawner.spawn(sensor_task(tsic_pin)).unwrap();

    // Spawn button input task with GPIO9
    spawner.spawn(button_task_spawn(brew_button)).unwrap();
}
