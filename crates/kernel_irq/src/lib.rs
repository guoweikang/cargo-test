pub fn init_irq() {
    println!("⚡ [IRQ] Initializing interrupt subsystem");
    
    #[cfg(SMP)]
    println!("⚡ [IRQ] SMP interrupt routing enabled");
    
    #[cfg(not(SMP))]
    println!("⚡ [IRQ] Single-core interrupt mode");
}
