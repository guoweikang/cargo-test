use kbuild_config::*;

#[cfg(CONFIG_NET)]
use network_utils;

pub fn demo() {
    println!("🎪 [DEMO] Demo Mixed Dependencies");
    println!("🎪 [DEMO] Log level = {}", CONFIG_LOG_LEVEL);
    println!("🎪 [DEMO] Max CPUs = {}", CONFIG_MAX_CPUS);
    println!("🎪 [DEMO] Default scheduler = {}", CONFIG_DEFAULT_SCHEDULER);
    
    #[cfg(CONFIG_NET)]
    {
        network_utils::init();
        println!("🎪 [DEMO] Network enabled via kbuild");
    }
    
    #[cfg(not(CONFIG_NET))]
    println!("🎪 [DEMO] Network disabled");
}
