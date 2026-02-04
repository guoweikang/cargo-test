use kernel_task::{self, Task};

pub fn schedule_init() {
    println!("🔄 [SCHEDULE] 调度器初始化");
    
    #[cfg(CONFIG_SMP)]
    {
        kernel_task::init_task_system();
        println!("🔄 [SCHEDULE] SMP 调度器已启用");
    }
    
    #[cfg(not(CONFIG_SMP))]
    println!("🔄 [SCHEDULE] 单核调度器");
    
    #[cfg(CONFIG_PREEMPT)]
    println!("🔄 [SCHEDULE] 抢占式调度已启用");
    
    #[cfg(not(CONFIG_PREEMPT))]
    println!("🔄 [SCHEDULE] 协作式调度");
}

pub fn schedule_on_cpu(task: &Task) {
    #[cfg(CONFIG_SMP)]
    println!("🔄 [SCHEDULE] 调度任务 {} 到 CPU {}", task.id, task.cpu);
    
    #[cfg(not(CONFIG_SMP))]
    println!("🔄 [SCHEDULE] 调度任务 {}", task.id);
}
