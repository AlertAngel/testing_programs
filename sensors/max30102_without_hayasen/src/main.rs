use esp_idf_svc::hal::{
    delay, i2c::*, peripherals::Peripherals, units::KiloHertz
};

use max3010x::{Max3010x, Led, SampleAveraging};
use log::{
    info, 
    error
};

fn main() {
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let i2c_config = I2cConfig::new().baudrate(KiloHertz(100).into());
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_config,
    ).unwrap();

    let mut sensor = Max3010x::new_max30102(i2c);
    let mut sensor = sensor.into_heart_rate().unwrap();

    sensor.set_sample_averaging(SampleAveraging::Sa4).unwrap();
    sensor.set_pulse_amplitude(Led::Led1, 15).unwrap();
    sensor.set_pulse_amplitude(Led::Led2, 15).unwrap();
    sensor.enable_fifo_rollover().unwrap();

    let mut data = [0; 3];

    loop {
        match sensor.read_fifo(&mut data) {
            Ok(_) => {
                info!("Red: {}, IR: {}, Green: {}", data[0], data[1], data[2]);
            }
            Err(e) => {
                error!("Error reading from sensor: {:?}", e);
            }
        }
        delay::Delay::new_default().delay_ms(500);
    }
}
