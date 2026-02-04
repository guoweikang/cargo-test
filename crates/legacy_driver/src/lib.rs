pub fn driver_init() {
    println!("🚗 [LEGACY] 传统驱动初始化");
    
    #[cfg(feature = "usb")]
    println!("🚗 [LEGACY] USB 驱动已加载");
    
    #[cfg(feature = "pci")]
    println!("🚗 [LEGACY] PCI 驱动已加载");
}
