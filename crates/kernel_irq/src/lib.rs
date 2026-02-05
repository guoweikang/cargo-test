pub fn init_irq() {
    println!("⚡ [IRQ] Initializing interrupt subsystem");
    
    #[cfg(CONFIG_SMP)]
    println!("⚡ [IRQ] SMP interrupt routing enabled");
    
    #[cfg(not(CONFIG_SMP))]
    println!("⚡ [IRQ] Single-core interrupt mode");
}
