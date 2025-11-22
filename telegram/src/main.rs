use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{EspWifi, ClientConfiguration, Configuration as WifiConfig};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use std::convert::TryInto;
use log::{error, info};

const SSID: &str = "";
const PASSWORD: &str = "";
const BOT_TOKEN: &str = "";
const CHAT_ID: &str = "";

fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn send_telegram_message(message: &str) -> Result<(), String> {
    use esp_idf_svc::sys::*;
    use std::ffi::CString;
    
    let encoded_msg = url_encode(message);
    
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
        BOT_TOKEN, CHAT_ID, encoded_msg
    );
    
    info!("Sending to: {}", url);
    
    unsafe {
        let url_c = CString::new(url).map_err(|e| e.to_string())?;
        
        let config = esp_http_client_config_t {
            url: url_c.as_ptr(),
            method: esp_http_client_method_t_HTTP_METHOD_GET,
            timeout_ms: 15000,
            crt_bundle_attach: Some(esp_crt_bundle_attach),
            buffer_size: 2048,
            buffer_size_tx: 1024,
            ..Default::default()
        };
        
        let client = esp_http_client_init(&config);
        
        if client.is_null() {
            return Err("Failed to initialize HTTP client".to_string());
        }
        
        let err = esp_http_client_perform(client);
        
        let status = esp_http_client_get_status_code(client);
        let content_length = esp_http_client_get_content_length(client);
        
        info!("HTTP Status: {}, Content Length: {}", status, content_length);
        
        esp_http_client_cleanup(client);
        
        if err != ESP_OK {
            return Err(format!("HTTP request failed with error code: {}", err));
        }
        
        if status >= 200 && status < 300 {
            info!("Message sent successfully!");
            Ok(())
        } else {
            Err(format!("HTTP error: {}", status))
        }
    }
}

fn main() -> ! {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    
    info!("Starting ESP32 Telegram Bot...");
    
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    
    let mut wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs)).unwrap();
    
    info!("Configuring WiFi...");
    
    wifi.set_configuration(&WifiConfig::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    }))
    .unwrap();
    
    wifi.start().unwrap();
    info!("WiFi starting...");
    wifi.connect().unwrap();
    
    info!("Connecting to WiFi...");
    while !wifi.is_connected().unwrap() {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    info!("WiFi connected successfully!");
    
    // Wait for connection to stabilize
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    let message = "Hello from ESP32 using Embedded Rust!";
    
    match send_telegram_message(message) {
        Ok(_) => info!("✅ Telegram notification sent successfully!"),
        Err(e) => error!("❌ Failed to send Telegram message: {}", e),
    }
    
    info!("Entering main loop...");
    
    loop {
        delay::Delay::new_default().delay_ms(1000);
    }
}
