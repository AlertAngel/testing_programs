fn main() -> ! {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    
    log::info!("=== MINIMAL TEST - BOARD ALIVE ===");
    
    let mut counter = 0;
    loop {
        counter += 1;
        log::info!("Counter: {}", counter);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
