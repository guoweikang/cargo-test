use kernel_task::{self, Task};

pub fn schedule_init() {
    println!("🔄 [SCHEDULE] Initializing scheduler");
    
    #[cfg(SMP)]
    {
        kernel_task::init_task_system();
        println!("🔄 [SCHEDULE] SMP scheduler enabled");
    }
    
    #[cfg(not(SMP))]
    println!("🔄 [SCHEDULE] Single-core scheduler");
    
    #[cfg(PREEMPT)]
    println!("🔄 [SCHEDULE] Preemptive scheduling enabled");
    
    #[cfg(not(PREEMPT))]
    println!("🔄 [SCHEDULE] Cooperative scheduling");
}

pub fn schedule_on_cpu(task: &Task) {
    #[cfg(SMP)]
    println!("🔄 [SCHEDULE] Scheduling task {} on CPU {}", task.id, task.cpu);
    
    #[cfg(not(SMP))]
    println!("🔄 [SCHEDULE] Scheduling task {}", task.id);
}
