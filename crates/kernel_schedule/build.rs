fn main() {
    // Declare CONFIG_* options for check-cfg lint
    println!("cargo:rustc-check-cfg=cfg(CONFIG_SMP)");
    println!("cargo:rustc-check-cfg=cfg(CONFIG_PREEMPT)");
}
