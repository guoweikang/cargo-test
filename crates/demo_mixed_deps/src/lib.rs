use kbuild_config::*;

pub fn demo() {
    println!("🎪 [DEMO] Demo Mixed Dependencies");
    println!("🎪 [DEMO] Log level = {}", CONFIG_LOG_LEVEL);
    println!("🎪 [DEMO] Max CPUs = {}", CONFIG_MAX_CPUS);
    println!("🎪 [DEMO] Default scheduler = {}", CONFIG_DEFAULT_SCHEDULER);
}
