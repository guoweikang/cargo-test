//! Kernel Network Subsystem
//! 
//! This module demonstrates kbuild integration with internal and external dependencies.

#[cfg(feature = "CONFIG_NET")]
use network_utils;

#[cfg(all(CONFIG_DEBUG, feature = "CONFIG_DEBUG"))]
use log;

/// Initialize the network subsystem
pub fn init() {
    #[cfg(feature = "CONFIG_NET")]
    {
        println!("🔧 [NET] Network subsystem initialized");
        network_utils::init();
        
        #[cfg(CONFIG_ASYNC)]
        println!("🔧 [NET] Async runtime enabled");
        
        #[cfg(not(CONFIG_ASYNC))]
        println!("🔧 [NET] Synchronous networking mode");
    }
    
    #[cfg(not(feature = "CONFIG_NET"))]
    {
        println!("🔧 [NET] Network subsystem disabled");
    }
}

/// Send network data
#[cfg(feature = "CONFIG_NET")]
pub fn send_data(data: &[u8]) {
    #[cfg(CONFIG_DEBUG)]
    {
        #[cfg(feature = "CONFIG_DEBUG")]
        log::debug!("Sending {} bytes", data.len());
    }
    
    network_utils::send_packet(data);
}

#[cfg(not(feature = "CONFIG_NET"))]
pub fn send_data(_data: &[u8]) {
    // No-op when network is disabled
}

/// Receive network data
#[cfg(feature = "CONFIG_NET")]
pub fn receive_data() -> Vec<u8> {
    #[cfg(CONFIG_DEBUG)]
    {
        #[cfg(feature = "CONFIG_DEBUG")]
        log::debug!("Receiving data");
    }
    
    network_utils::receive_packet()
}

#[cfg(not(feature = "CONFIG_NET"))]
pub fn receive_data() -> Vec<u8> {
    Vec::new()
}

/// Test network operations
pub fn test_network() {
    #[cfg(feature = "CONFIG_NET")]
    {
        println!("🧪 [NET] Testing network operations");
        
        let test_data = b"Hello, network!";
        send_data(test_data);
        let received = receive_data();
        
        println!("✅ [NET] Network test complete ({} bytes received)", received.len());
    }
    
    #[cfg(not(feature = "CONFIG_NET"))]
    {
        println!("⚠️  [NET] Network test skipped (network disabled)");
    }
}
