# Fall Detection

This program detects falls using data from an accelerometer sensor. It processes the sensor data to identify sudden changes in acceleration that may indicate a fall event.

## Output 

### Serial monitor 

![sm](./fall_detection.gif)

## Code 

```rust
use esp_idf_svc::hal::{
    i2c::*,
    delay::Delay,
    prelude::*,
    peripherals::Peripherals,
};

use hayasen::mpu9250_hayasen;
use log::{
    info,
    warn,
    error
};

const DEVICE_ADDR: u8 = 0x68; // I2C address for MPU9250 
const FREE_FALL_THRESHOLD: f32 = 0.5; // g 
const IMPACT_THREASHOLD: f32 = 2.5; // g 
const GYRO_THRESHOLD: f32 = 150.0; // °/s

fn main() {
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let delay = Delay::new_default();

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    let i2c_config = I2cConfig::new().baudrate(100.kHz().into());

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        sda,
        scl,
        &i2c_config,
    )
        .unwrap();

    let mut sensor = mpu9250_hayasen::create_default(i2c,DEVICE_ADDR).unwrap();

    loop {
        match mpu9250_hayasen::read_all(&mut sensor) {
            Ok((temp, accel, gyro)) => {
                info!(
                    "\n\nTemp: {:.2} °C\nAccel: x: {:.2} g, y: {:.2} g, z: {:.2} g\nGyro: x: {:.2} °/s, y: {:.2} °/s, z: {:.2} °/s\n",
                    temp,
                    accel[0], accel[1], accel[2],
                    gyro[0], gyro[1], gyro[2]
                );

                let accel_mag = (accel[0].powi(2) + accel[1].powi(2) + accel[2].powi(2)).sqrt();
                if accel_mag < FREE_FALL_THRESHOLD {
                    warn!("Free fall detected!");
                } else if accel_mag > IMPACT_THREASHOLD {
                    warn!("Impact detected!");
                } else if gyro.iter().any(|&g| g.abs() > GYRO_THRESHOLD) {
                    warn!("High rotation detected!");
                }

            },
            Err(e) => {
                error!("Error reading sensor data: {:?}", e);
            }
        }


        delay.delay_ms(500);
    }
}
```
