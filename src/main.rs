use kernel_schedule::{schedule_init, schedule_on_cpu};
use kernel_task::create_task;
use legacy_driver::driver_init;

#[cfg(CONFIG_NET)]
use kernel_net::net_init;

#[cfg(CONFIG_NET)]
use demo_mixed_deps;

fn main() {
    println!("🚀 ============================================");
    println!("🚀  Cargo-Kbuild Demo");
    println!("🚀 ============================================\n");

    // Initialize scheduler
    schedule_init();
    
    println!();
    
    // Create tasks
    #[cfg(CONFIG_SMP)]
    {
        let task1 = create_task(1, 0);
        let task2 = create_task(2, 1);
        schedule_on_cpu(&task1);
        schedule_on_cpu(&task2);
    }
    
    #[cfg(not(CONFIG_SMP))]
    {
        let task1 = create_task(1);
        schedule_on_cpu(&task1);
    }
    
    println!();
    
    // Initialize network subsystem (new)
    #[cfg(CONFIG_NET)]
    {
        net_init();
        println!();
    }
    
    // Demo mixed dependencies with config constants
    #[cfg(CONFIG_NET)]
    {
        demo_mixed_deps::demo();
        println!();
    }
    
    // Initialize legacy driver
    driver_init();
    
    println!("\n🎉 ============================================");
    println!("🎉  System initialization complete");
    println!("🎉 ============================================");
}
