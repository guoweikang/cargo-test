//! Legacy Driver Module
//! 
//! This module demonstrates traditional Cargo features (not kbuild-enabled).
//! Used for comparison purposes.

/// Initialize the legacy driver
pub fn init() {
    println!("🔧 [LEGACY] Initialize legacy driver");
    
    #[cfg(feature = "usb")]
    init_usb();
    
    #[cfg(feature = "pci")]
    init_pci();
}

#[cfg(feature = "usb")]
fn init_usb() {
    println!("🔧 [LEGACY] USB support enabled");
}

#[cfg(feature = "pci")]
fn init_pci() {
    println!("🔧 [LEGACY] PCI support enabled");
}

/// Probe for devices
pub fn probe() {
    println!("🔍 [LEGACY] Probing for devices");
    
    #[cfg(feature = "usb")]
    println!("🔍 [LEGACY] Found USB device");
    
    #[cfg(feature = "pci")]
    println!("🔍 [LEGACY] Found PCI device");
}
