pub fn init() {
    println!("🔧 [NETWORK_UTILS] 初始化网络工具");
    
    #[cfg(CONFIG_ASYNC)]
    println!("🔧 [NETWORK_UTILS] 异步网络支持已启用");
}
