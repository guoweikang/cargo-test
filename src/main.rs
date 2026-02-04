//! Cargo-Kbuild MVP Demo Application
//! 
//! This application demonstrates the Kconfig-Cargo integration system.

use kernel_schedule;

#[cfg(feature = "CONFIG_NET")]
use kernel_net;

fn main() {
    print_banner();
    
    // Initialize all subsystems in dependency order
    println!("\n📦 Initializing subsystems...\n");
    
    // Core scheduler (initializes task and IRQ internally)
    kernel_schedule::init();
    
    // Network subsystem (if enabled)
    #[cfg(feature = "CONFIG_NET")]
    {
        kernel_net::init();
        
        // Test network operations
        kernel_net::test_network();
    }
    
    #[cfg(not(feature = "CONFIG_NET"))]
    {
        println!("⚠️  [NET] Network subsystem not configured");
    }
    
    // Run the scheduler
    println!("\n🎯 Running system...\n");
    kernel_schedule::run();
    
    print_footer();
    print_config_summary();
}

fn print_banner() {
    println!("🚀 ============================================");
    println!("🚀  Cargo-Kbuild MVP Demo");
    println!("🚀 ============================================");
}

fn print_footer() {
    println!("\n🎉 ============================================");
    println!("🎉  System initialization complete");
    println!("🎉 ============================================");
}

fn print_config_summary() {
    println!("\n📋 Configuration Summary:");
    
    #[cfg(CONFIG_SMP)]
    println!("   ✅ CONFIG_SMP: Enabled");
    #[cfg(not(CONFIG_SMP))]
    println!("   ❌ CONFIG_SMP: Disabled");
    
    #[cfg(CONFIG_PREEMPT)]
    println!("   ✅ CONFIG_PREEMPT: Enabled");
    #[cfg(not(CONFIG_PREEMPT))]
    println!("   ❌ CONFIG_PREEMPT: Disabled");
    
    #[cfg(feature = "CONFIG_NET")]
    println!("   ✅ CONFIG_NET: Enabled");
    #[cfg(not(feature = "CONFIG_NET"))]
    println!("   ❌ CONFIG_NET: Disabled");
    
    #[cfg(CONFIG_ASYNC)]
    println!("   ✅ CONFIG_ASYNC: Enabled");
    #[cfg(not(CONFIG_ASYNC))]
    println!("   ❌ CONFIG_ASYNC: Disabled");
    
    #[cfg(CONFIG_DEBUG)]
    println!("   ✅ CONFIG_DEBUG: Enabled");
    #[cfg(not(CONFIG_DEBUG))]
    println!("   ❌ CONFIG_DEBUG: Disabled");
    
    println!();
}
