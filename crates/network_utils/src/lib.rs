pub fn init() {
    println!("🔧 [NETWORK_UTILS] Initializing network utilities");
    
    #[cfg(CONFIG_ASYNC)]
    println!("🔧 [NETWORK_UTILS] Async network support enabled");
}
