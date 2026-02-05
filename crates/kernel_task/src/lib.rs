#[cfg(CONFIG_SMP)]
use kernel_irq;

pub struct Task {
    pub id: u32,
    #[cfg(CONFIG_SMP)]
    pub cpu: u32,
}

#[cfg(CONFIG_SMP)]
pub fn create_task(id: u32, cpu: u32) -> Task {
    println!("📋 [TASK] Creating task {} (bound to CPU {})", id, cpu);
    Task { id, cpu }
}

#[cfg(not(CONFIG_SMP))]
pub fn create_task(id: u32) -> Task {
    println!("📋 [TASK] Creating task {}", id);
    Task { id }
}

pub fn init_task_system() {
    #[cfg(CONFIG_SMP)]
    {
        kernel_irq::init_irq();
        println!("📋 [TASK] SMP task system initialized");
    }
    
    #[cfg(not(CONFIG_SMP))]
    println!("📋 [TASK] Single-core task system initialized");
}
