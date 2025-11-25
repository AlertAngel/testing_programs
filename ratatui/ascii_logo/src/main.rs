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
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    widgets::Paragraph,
    Frame, Terminal
};
use static_cell::StaticCell;

static SPI_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

const LOGO: &str = r#"
 █████╗ ██╗     ███████╗██████╗ ████████╗
██╔══██╗██║     ██╔════╝██╔══██╗╚══██╔══╝
███████║██║     █████╗  ██████╔╝   ██║   
██╔══██║██║     ██╔══╝  ██╔══██╗   ██║   
██║  ██║███████╗███████╗██║  ██║   ██║   
╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝   ╚═╝  
 █████╗ ███╗   ██╗ ██████╗ ███████╗██╗     
██╔══██╗████╗  ██║██╔════╝ ██╔════╝██║     
███████║██╔██╗ ██║██║  ███╗█████╗  ██║     
██╔══██║██║╚██╗██║██║   ██║██╔══╝  ██║     
██║  ██║██║ ╚████║╚██████╔╝███████╗███████╗
╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝╚══════╝
"#;

fn main() {
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

    log::info!("Orientation set. DIslay Cleared");

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());

    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(draw_logo).unwrap();

    log::info!("Logo Drawn Successfully");

    loop {
        delay.delay_ms(1000);
    }
}

fn draw_logo(frame: &mut Frame) {
    let layout = Layout::vertical([
        Constraint::Percentage(20),
        Constraint::Min(10),
        Constraint::Percentage(10)
    ])
        .split(frame.area());

    let logo_widget = Paragraph::new(LOGO)
        .style(Style::default().fg(Color::LightYellow).bold())
        .alignment(Alignment::Center);

    frame.render_widget(logo_widget, layout[1]);
}
