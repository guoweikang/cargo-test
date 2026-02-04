//! Kernel Task Management Module
//! 
//! This module demonstrates kbuild integration with dependencies on kernel_irq.

use kernel_irq;

/// Initialize the task management subsystem
pub fn init() {
    println!("🔧 [TASK] Initialize task management");
    
    #[cfg(CONFIG_SMP)]
    init_smp_scheduler();
    
    #[cfg(not(CONFIG_SMP))]
    init_simple_scheduler();
    
    #[cfg(CONFIG_PREEMPT)]
    enable_preemption();
}

#[cfg(CONFIG_SMP)]
fn init_smp_scheduler() {
    println!("🔧 [TASK] Multi-core task scheduling enabled");
}

#[cfg(not(CONFIG_SMP))]
fn init_simple_scheduler() {
    println!("🔧 [TASK] Single-core task scheduling enabled");
}

#[cfg(CONFIG_PREEMPT)]
fn enable_preemption() {
    println!("🔧 [TASK] Task preemption enabled");
}

/// Create a new task
pub fn create_task(name: &str) {
    println!("📋 [TASK] Creating task: {}", name);
    
    #[cfg(CONFIG_SMP)]
    println!("📋 [TASK] Task will be scheduled on any available CPU");
    
    #[cfg(not(CONFIG_SMP))]
    println!("📋 [TASK] Task will run on single CPU");
}

/// Schedule tasks
pub fn schedule() {
    #[cfg(CONFIG_PREEMPT)]
    {
        println!("⏰ [TASK] Preemptive scheduling");
        // Simulate interrupt-driven scheduling
        kernel_irq::handle_interrupt(0);
    }
    
    #[cfg(not(CONFIG_PREEMPT))]
    {
        println!("⏰ [TASK] Cooperative scheduling");
    }
}
