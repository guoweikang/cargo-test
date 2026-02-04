pub fn init_irq() {
    println!("⚡ [IRQ] 中断子系统初始化");
    
    #[cfg(CONFIG_SMP)]
    println!("⚡ [IRQ] SMP 中断路由已启用");
    
    #[cfg(not(CONFIG_SMP))]
    println!("⚡ [IRQ] 单核中断模式");
}
