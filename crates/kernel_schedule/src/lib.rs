use kernel_task::{self, Task};

pub fn schedule_init() {
    println!("🔄 [SCHEDULE] Initializing scheduler");
    
    #[cfg(CONFIG_SMP)]
    {
        kernel_task::init_task_system();
        println!("🔄 [SCHEDULE] SMP scheduler enabled");
    }
    
    #[cfg(not(CONFIG_SMP))]
    println!("🔄 [SCHEDULE] Single-core scheduler");
    
    #[cfg(CONFIG_PREEMPT)]
    println!("🔄 [SCHEDULE] Preemptive scheduling enabled");
    
    #[cfg(not(CONFIG_PREEMPT))]
    println!("🔄 [SCHEDULE] Cooperative scheduling");
}

pub fn schedule_on_cpu(task: &Task) {
    #[cfg(CONFIG_SMP)]
    println!("🔄 [SCHEDULE] Scheduling task {} on CPU {}", task.id, task.cpu);
    
    #[cfg(not(CONFIG_SMP))]
    println!("🔄 [SCHEDULE] Scheduling task {}", task.id);
}
