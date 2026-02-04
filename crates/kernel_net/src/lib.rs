use network_utils;

pub fn net_init() {
    println!("🌐 [NET] 网络子系统初始化");
    
    network_utils::init();
    println!("🌐 [NET] 网络工具库已加载");
    
    #[cfg(CONFIG_LOGGING)]
    {
        // In a real scenario, this would use log crate
        println!("📝 [NET] 日志系统已启用");
    }
}
