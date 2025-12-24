use esp_idf_svc::hal::{
    gpio::{PinDriver, Output},
    prelude::*,
};
use esp_idf_sys as sys;
use log::info;
use std::time::Duration;

fn main() {
    sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // Setup LED on GPIO 2 (built-in LED)
    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();

    info!("=== Light Sleep Test ===");
    info!("LED ON = Active Mode");
    info!("LED OFF = Light Sleep Mode");

    loop {
        // ACTIVE MODE - LED ON
        led.set_high().unwrap();
        info!("🔆 ACTIVE MODE - LED ON");
        info!("Doing work for 5 seconds...");
        
        for i in 1..=5 {
            std::thread::sleep(Duration::from_secs(1));
            info!("  Active: {} sec", i);
        }

        // Entering LIGHT SLEEP - LED OFF
        led.set_low().unwrap();
        info!("💤 ENTERING LIGHT SLEEP - LED OFF");
        info!("Sleeping for 10 seconds...");

        // Configure light sleep for 10 seconds
        unsafe {
            // Enable timer wakeup (10 seconds = 10,000,000 microseconds)
            sys::esp_sleep_enable_timer_wakeup(10 * 1_000_000);
            
            // Enter light sleep (peripherals stay powered)
            info!("Entering light sleep now...");
            sys::esp_light_sleep_start();
            
            info!("⏰ WOKE UP from light sleep!");
        }

        // Small delay before next cycle
        std::thread::sleep(Duration::from_millis(500));
    }
}
