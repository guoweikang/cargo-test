use kernel_irq;

pub struct Task {
    pub id: u32,
    #[cfg(CONFIG_SMP)]
    pub cpu: u32,
}

#[cfg(CONFIG_SMP)]
pub fn create_task(id: u32, cpu: u32) -> Task {
    println!("📋 [TASK] 创建任务 {} (绑定到 CPU {})", id, cpu);
    Task { id, cpu }
}

#[cfg(not(CONFIG_SMP))]
pub fn create_task(id: u32) -> Task {
    println!("📋 [TASK] 创建任务 {}", id);
    Task { id }
}

pub fn init_task_system() {
    #[cfg(CONFIG_SMP)]
    {
        kernel_irq::init_irq();
        println!("📋 [TASK] SMP 任务系统初始化");
    }
    
    #[cfg(not(CONFIG_SMP))]
    println!("📋 [TASK] 单核任务系统初始化");
}
