use network_utils;

pub fn net_init() {
    println!("🌐 [NET] Initializing network subsystem");
    
    network_utils::init();
    println!("🌐 [NET] Network utilities loaded");
    
    #[cfg(LOGGING)]
    {
        // In a real scenario, this would use log crate
        println!("📝 [NET] Logging system enabled");
    }
}
