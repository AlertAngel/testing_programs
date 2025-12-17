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

    let reset = PinDriver::output(peripherals.pins.gpio8).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio3).unwrap();

    let mut delay = Delay::new_default();

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio10,
        peripherals.pins.gpio6,
        None::<AnyInputPin>,
        &esp_idf_svc::hal::spi::config::DriverConfig::default()
            .dma(Dma::Auto(320 * 240 * 2 + 8)),
    )
    .unwrap();

    log::info!("SPI initialized");

    let spi = SpiDeviceDriver::new(
        spi_driver,
        Some(peripherals.pins.gpio9),
        &Config::new().baudrate(Hertz(26_000_000)),
    )
    .unwrap();

    let buffer = SPI_BUFFER.init([0; 512]);
    let di = SpiInterface::new(spi, dc, buffer);

    let mut display = Builder::new(ILI9342CRgb565, di)
        .reset_pin(reset)
        .init(&mut delay)
        .unwrap();

    log::info!("Display initialized");

    display
        .set_orientation(Orientation::default().rotate(Rotation::Deg270))
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    log::info!("Orientation set. Display Cleared");

    // Check available heap before creating backend
    log::info!("Free heap before backend: {} bytes", unsafe {
        esp_idf_svc::sys::esp_get_free_heap_size()
    });

    let backend_result = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    
    log::info!("Backend created");

    let mut terminal = match Terminal::new(backend_result) {
        Ok(t) => {
            log::info!("Terminal created successfully");
            t
        }
        Err(e) => {
            log::error!("Failed to create terminal: {:?}", e);
            panic!("Terminal creation failed");
        }
    };

    let mut app = App::new();
    
    // Pre-populate with some data so first frame isn't empty
    for _ in 0..20 {
        app.update();
    }

    log::info!("Starting draw loop");

    loop {
        app.update();
        
        match terminal.draw(|frame| app.draw(frame)) {
            Ok(_) => {
                log::info!("Frame drawn successfully");
            }
            Err(e) => {
                log::error!("Draw failed: {:?}", e);
            }
        }
        
        delay.delay_ms(100);
    }
}
