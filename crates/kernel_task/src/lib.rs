#[cfg(SMP)]
use kernel_irq;

pub struct Task {
    pub id: u32,
    #[cfg(SMP)]
    pub cpu: u32,
}

#[cfg(SMP)]
pub fn create_task(id: u32, cpu: u32) -> Task {
    println!("📋 [TASK] Creating task {} (bound to CPU {})", id, cpu);
    Task { id, cpu }
}

#[cfg(not(SMP))]
pub fn create_task(id: u32) -> Task {
    println!("📋 [TASK] Creating task {}", id);
    Task { id }
}

pub fn init_task_system() {
    #[cfg(SMP)]
    {
        kernel_irq::init_irq();
        println!("📋 [TASK] SMP task system initialized");
    }
    
    #[cfg(not(SMP))]
    println!("📋 [TASK] Single-core task system initialized");
}
