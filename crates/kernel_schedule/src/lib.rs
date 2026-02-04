//! Kernel Scheduler Module
//! 
//! This module demonstrates kbuild integration with multiple dependencies.

use kernel_task;
use kernel_irq;

/// Initialize the scheduler
pub fn init() {
    println!("🔧 [SCHEDULE] Initialize scheduler");
    
    // Initialize dependencies first
    kernel_irq::init();
    kernel_task::init();
    
    #[cfg(CONFIG_SMP)]
    init_multicore_scheduler();
    
    #[cfg(not(CONFIG_SMP))]
    init_singlecore_scheduler();
}

#[cfg(CONFIG_SMP)]
fn init_multicore_scheduler() {
    println!("🔧 [SCHEDULE] Multi-core scheduler initialized");
    println!("🔧 [SCHEDULE] Load balancing enabled");
}

#[cfg(not(CONFIG_SMP))]
fn init_singlecore_scheduler() {
    println!("🔧 [SCHEDULE] Single-core scheduler initialized");
}

/// Run the scheduler
pub fn run() {
    println!("🚀 [SCHEDULE] Running scheduler");
    
    // Create some example tasks
    kernel_task::create_task("init");
    kernel_task::create_task("worker-1");
    kernel_task::create_task("worker-2");
    
    // Schedule them
    kernel_task::schedule();
}
