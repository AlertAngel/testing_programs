use esp_idf_svc::hal::{gpio::PinDriver, peripherals::Peripherals, delay::Delay};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();
    let delay = Delay::new_default();

    let button = PinDriver::input(peripherals.pins.gpio26).unwrap();

    let mut prev_state = false;

    loop {
        let is_pressed = button.is_high();

        if is_pressed != prev_state {
            prev_state = is_pressed;

            if is_pressed {
                led.set_high().unwrap();
                log::info!("Button pressed, LED ON");
            } else {
                led.set_low().unwrap();
                log::info!("Button released, LED OFF");
            }
        }

        delay.delay_ms(50); // debounce delay
    }
}
