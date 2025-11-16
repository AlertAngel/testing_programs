# Chart

This program shows how to create a chart in Embedded displays using `ratatui`.

Chart self updates and moves

## Output 

![o](./chart.gif)

## Code

```rust
use std::error::Error;
use std::collections::VecDeque;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use esp_idf_svc::hal::{
    delay::Delay, gpio::{AnyInputPin, PinDriver}, peripherals::Peripherals, spi::{
        Dma, SpiDeviceDriver, SpiDriver, config::Config
    }, units::Hertz
};

use mipidsi::{interface::SpiInterface, models::ILI9342CRgb565, Builder, options::{Orientation, Rotation}};

use mousefood::{
    EmbeddedBackend,
    EmbeddedBackendConfig
};

use ratatui::{
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
    Terminal, Frame
};

use static_cell::StaticCell;
use rand::Rng;

static SPI_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

struct App {
    data: VecDeque<(f64, f64)>,
    x_offset: f64,
}

impl App {
    fn new() -> Self {
        Self {
            data: VecDeque::with_capacity(50),
            x_offset: 0.0,
        }
    }

    fn update(&mut self) {
        let mut rng = rand::rng();
        
        // Add new data point
        let y = rng.random_range(0.0..100.0);
        self.data.push_back((self.x_offset, y));
        
        // Remove old data points if we have more than 50
        if self.data.len() > 50 {
            self.data.pop_front();
        }
        
        // Increment x position
        self.x_offset += 0.2;
    }

    fn draw(&self, frame: &mut Frame) {
        // Convert VecDeque to Vec for the dataset
        let data: Vec<(f64, f64)> = self.data.iter().copied().collect();
        
        // Calculate dynamic x-axis bounds (show last 10 units)
        let x_max = self.x_offset;
        let x_min = (x_max - 10.0).max(0.0);
        
        let dataset = Dataset::default()
            .name("Real-time Data")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::LightYellow))
            .data(&data);

        let chart = Chart::new(vec![dataset])
            .block(Block::bordered().title("Animated Chart"))
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([x_min, x_max])
                    .labels([
                        format!("{:.1}", x_min),
                        format!("{:.1}", (x_min + x_max) / 2.0),
                        format!("{:.1}", x_max),
                    ])
            )
            .y_axis(
                Axis::default()
                    .title("Value")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 100.0])
                    .labels(["0", "50", "100"])
            );

        frame.render_widget(chart, frame.area());
    }
}

fn main() -> ! {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let reset = PinDriver::output(peripherals.pins.gpio4).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio2).unwrap();
    let mut delay = Delay::new_default();

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        None::<AnyInputPin>,
        &esp_idf_svc::hal::spi::config::DriverConfig::default()
            .dma(Dma::Channel1(320 * 240 * 2 + 8)),
    ).unwrap();

    let spi = SpiDeviceDriver::new(
        spi_driver,
        Some(peripherals.pins.gpio15),
        &Config::new().baudrate(Hertz(26_000_000))
    ).unwrap();

    let buffer = SPI_BUFFER.init([0; 512]);
    let di = SpiInterface::new(spi, dc, buffer);

    let mut display = Builder::new(ILI9342CRgb565, di)
        .reset_pin(reset)
        .init(&mut delay)
        .map_err(|_| Box::<dyn Error>::from("Display Init Failed"))
        .unwrap();

    display
        .set_orientation(Orientation::default().rotate(Rotation::Deg270))
        .map_err(|_| Box::<dyn Error>::from("Set Orientation Failed"))
        .unwrap();

    display
        .clear(Rgb565::BLACK)
        .map_err(|_| Box::<dyn Error>::from("Clear Display Failed"))
        .unwrap();

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend)
        .map_err(|_| Box::<dyn Error>::from("Terminal creation failed"))
        .unwrap();

    let mut app = App::new();

    loop {
        app.update();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        delay.delay_ms(100); // Update every 100ms for smooth animation
    }
}
```
