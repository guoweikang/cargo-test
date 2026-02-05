use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
    #[serde(default)]
    features: HashMap<String, Vec<String>>,
    // Note: dependencies field kept for potential future feature validation
    #[serde(default)]
    #[allow(dead_code)]
    dependencies: HashMap<String, toml::Value>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    #[serde(default)]
    metadata: Metadata,
}

#[derive(Debug, Deserialize, Default)]
struct Metadata {
    #[serde(default)]
    kbuild: KbuildMetadata,
}

#[derive(Debug, Deserialize, Default)]
struct KbuildMetadata {
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug)]
struct CrateInfo {
    name: String,
    // Note: path field kept for potential future features (e.g., detailed error reporting)
    #[allow(dead_code)]
    path: PathBuf,
    has_kbuild: bool,
    features: HashMap<String, Vec<String>>,
}

impl CrateInfo {
    fn is_kbuild_enabled(&self) -> bool {
        self.has_kbuild || self.features.keys().any(|f| f.starts_with("CONFIG_"))
    }
}

#[derive(Debug)]
struct Workspace {
    // Note: root field kept for potential future features (e.g., relative path resolution)
    #[allow(dead_code)]
    root: PathBuf,
    crates: Vec<CrateInfo>,
}

impl Workspace {
    fn new(root: PathBuf) -> Result<Self, String> {
        let mut crates = Vec::new();
        
        // Read workspace Cargo.toml
        let workspace_toml_path = root.join("Cargo.toml");
        let workspace_toml_content = fs::read_to_string(&workspace_toml_path)
            .map_err(|e| format!("Failed to read workspace Cargo.toml: {}", e))?;
        
        let workspace_toml: toml::Value = toml::from_str(&workspace_toml_content)
            .map_err(|e| format!("Failed to parse workspace Cargo.toml: {}", e))?;
        
        // Get workspace members
        let members = workspace_toml
            .get("workspace")
            .and_then(|w| w.get("members"))
            .and_then(|m| m.as_array())
            .ok_or("No workspace members found")?;
        
        // Parse each member crate
        for member in members {
            let member_path = member.as_str().ok_or("Invalid member path")?;
            let crate_path = root.join(member_path);
            
            if let Ok(crate_info) = Self::parse_crate(&crate_path) {
                crates.push(crate_info);
            }
        }
        
        Ok(Workspace { root, crates })
    }
    
    fn parse_crate(crate_path: &Path) -> Result<CrateInfo, String> {
        let cargo_toml_path = crate_path.join("Cargo.toml");
        let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
            .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path.display(), e))?;
        
        let cargo_toml: CargoToml = toml::from_str(&cargo_toml_content)
            .map_err(|e| format!("Failed to parse {}: {}", cargo_toml_path.display(), e))?;
        
        Ok(CrateInfo {
            name: cargo_toml.package.name.clone(),
            path: crate_path.to_path_buf(),
            has_kbuild: cargo_toml.package.metadata.kbuild.enabled,
            features: cargo_toml.features,
        })
    }
    
    // Note: find_crate method kept for potential future features (e.g., dependency graph analysis)
    #[allow(dead_code)]
    fn find_crate(&self, name: &str) -> Option<&CrateInfo> {
        self.crates.iter().find(|c| c.name == name)
    }
}

/// Check if a dependency package supports kbuild
/// Note: Function kept for potential future validation features
#[allow(dead_code)]
fn is_dependency_kbuild_enabled(workspace: &Workspace, pkg_name: &str) -> bool {
    if let Some(dep_crate) = workspace.find_crate(pkg_name) {
        // Method 1: Check metadata.kbuild.enabled
        if dep_crate.has_kbuild {
            return true;
        }
        
        // Method 2: Check if it has CONFIG_* prefixed features
        if dep_crate.features.keys().any(|f| f.starts_with("CONFIG_")) {
            return true;
        }
    }
    
    false
}

/// Validate features for all kbuild-enabled crates
fn validate_features(workspace: &Workspace) -> Result<(), String> {
    println!("🔍 Validating feature dependencies...\n");
    
    // 1. Build a set of kbuild-enabled packages for performance
    let kbuild_packages: HashSet<String> = workspace
        .crates
        .iter()
        .filter(|c| c.is_kbuild_enabled())
        .map(|c| c.name.clone())
        .collect();
    
    // 2. Build a set of all workspace packages
    let workspace_packages: HashSet<String> = workspace
        .crates
        .iter()
        .map(|c| c.name.clone())
        .collect();
    
    // 3. Validate each kbuild-enabled crate's features
    for crate_info in workspace.crates.iter().filter(|c| c.is_kbuild_enabled()) {
        for (feature_name, deps) in &crate_info.features {
            // Only check CONFIG_* features
            if !feature_name.starts_with("CONFIG_") {
                continue;
            }
            
            for dep in deps {
                // Check if sub-feature is specified
                if let Some((pkg_name, sub_feature)) = dep.split_once('/') {
                    // Key decision: Does the dependency support kbuild?
                    if kbuild_packages.contains(pkg_name) {
                        // ❌ Error: kbuild-enabled workspace crate cannot specify sub-feature
                        return Err(format!(
                            "❌ Error in crate '{}':\n\
                             \n\
                             Feature '{}' specifies sub-feature: '{}'\n\
                             \n\
                             Dependency '{}' is kbuild-enabled:\n\
                             - It reads CONFIG_* from .config directly\n\
                             - Cannot be controlled by parent crate\n\
                             \n\
                             Solution:\n\
                             1. Change to: {} = [\"{}\"]\n\
                             2. Enable {} in .config file\n\
                             \n\
                             Note: Third-party crates (e.g., log/std, tokio/rt) are allowed sub-features.\n",
                            crate_info.name,
                            feature_name,
                            dep,
                            pkg_name,
                            feature_name, pkg_name,
                            sub_feature
                        ));
                    } else if workspace_packages.contains(pkg_name) {
                        // ℹ️ Info: Non-kbuild workspace crate - sub-feature allowed
                        eprintln!(
                            "ℹ️  '{}' is not kbuild-enabled, sub-feature allowed: {}\n",
                            pkg_name, dep
                        );
                    } else {
                        // ℹ️ Info: Third-party library - sub-feature allowed
                        eprintln!(
                            "ℹ️  '{}' is third-party, sub-feature allowed: {}\n",
                            pkg_name, dep
                        );
                    }
                }
            }
        }
    }
    
    println!("✅ Feature validation passed!\n");
    Ok(())
}

/// Collect all CONFIG_* names from .config file
fn collect_all_configs_from_file(config: &HashMap<String, String>) -> HashSet<String> {
    let mut configs = HashSet::new();
    
    for key in config.keys() {
        if key.starts_with("CONFIG_") {
            configs.insert(key.clone());
        }
    }
    
    configs
}

/// Collect all CONFIG_* feature names from workspace crates (for validation only)
fn collect_all_configs(workspace: &Workspace) -> HashSet<String> {
    let mut configs = HashSet::new();
    
    for crate_info in workspace.crates.iter().filter(|c| c.is_kbuild_enabled()) {
        for feature_name in crate_info.features.keys() {
            if feature_name.starts_with("CONFIG_") {
                configs.insert(feature_name.clone());
            }
        }
    }
    
    configs
}

/// Generate .cargo/config.toml with check-cfg declarations
fn generate_cargo_config(workspace_root: &Path, configs: &HashSet<String>) -> Result<(), String> {
    let cargo_dir = workspace_root.join(".cargo");
    fs::create_dir_all(&cargo_dir)
        .map_err(|e| format!("Failed to create .cargo directory: {}", e))?;
    
    let config_path = cargo_dir.join("config.toml");
    
    let mut content = String::from("# Auto-generated by cargo-kbuild\n");
    content.push_str("# This file declares all CONFIG_* conditional compilation flags\n");
    content.push_str("# Run 'cargo-kbuild build' to regenerate this file\n");
    content.push_str("# DO NOT commit this file to git\n\n");
    content.push_str("[build]\n");
    content.push_str("rustflags = [\n");
    
    let mut sorted_configs: Vec<_> = configs.iter().collect();
    sorted_configs.sort();
    
    for config in sorted_configs {
        content.push_str(&format!("    \"--check-cfg=cfg({})\",\n", config));
    }
    
    content.push_str("]\n");
    
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write .cargo/config.toml: {}", e))?;
    
    println!("✅ Generated .cargo/config.toml with {} CONFIG_* declarations", configs.len());
    Ok(())
}

/// Parse .config file
fn parse_config(config_path: &Path) -> Result<HashMap<String, String>, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read .config: {}", e))?;
    
    let mut config = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            config.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    
    Ok(config)
}

/// Generate features based on .config
fn generate_features(config: &HashMap<String, String>) -> Vec<String> {
    let mut features = Vec::new();
    
    for (key, value) in config {
        if key.starts_with("CONFIG_") && (value == "y" || value == "m") {
            features.push(key.clone());
        }
    }
    
    features
}

/// Generate config.rs file with constants
fn generate_config_rs(workspace_root: &Path, config: &HashMap<String, String>) -> Result<(), String> {
    // Create target/kbuild directory
    let target_dir = workspace_root.join("target/kbuild");
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target/kbuild: {}", e))?;
    
    let config_rs_path = target_dir.join("config.rs");
    
    // Generate config.rs content
    let mut content = String::new();
    content.push_str("// Auto-generated by cargo-kbuild from .config\n");
    content.push_str("// DO NOT EDIT MANUALLY\n\n");
    
    // Process each config value
    for (key, value) in config {
        if !key.starts_with("CONFIG_") {
            continue;
        }
        
        // Skip boolean configs (y/n) as they're handled via --cfg
        if value == "y" || value == "n" || value == "m" {
            continue;
        }
        
        // Try to parse as integer
        if let Ok(int_val) = value.parse::<i32>() {
            content.push_str(&format!("#[allow(dead_code)]\n"));
            content.push_str(&format!("pub const {}: i32 = {};\n\n", key, int_val));
        }
        // Check if it's a string (starts and ends with quotes)
        else if value.starts_with('"') && value.ends_with('"') {
            let str_val = &value[1..value.len()-1]; // Remove quotes
            content.push_str(&format!("#[allow(dead_code)]\n"));
            content.push_str(&format!("pub const {}: &str = \"{}\";\n\n", key, str_val));
        }
        // Otherwise treat as usize
        else if let Ok(uint_val) = value.parse::<usize>() {
            content.push_str(&format!("#[allow(dead_code)]\n"));
            content.push_str(&format!("pub const {}: usize = {};\n\n", key, uint_val));
        }
    }
    
    // Write the file
    fs::write(&config_rs_path, content)
        .map_err(|e| format!("Failed to write config.rs: {}", e))?;
    
    println!("📝 Generated config.rs at: {}", config_rs_path.display());
    
    Ok(())
}

/// Build command
fn build(workspace_root: &Path, config_path: &Path) -> Result<(), String> {
    println!("🔨 Starting cargo-kbuild build...\n");
    
    // Parse workspace
    let workspace = Workspace::new(workspace_root.to_path_buf())?;
    
    // Parse .config first to get all CONFIG_* options
    let config = parse_config(config_path)?;
    
    // Collect all CONFIG_* names from .config file and generate .cargo/config.toml
    let all_configs = collect_all_configs_from_file(&config);
    generate_cargo_config(workspace_root, &all_configs)?;
    println!();
    
    // Validate features
    validate_features(&workspace)?;
    
    // Generate config.rs file with constants
    generate_config_rs(workspace_root, &config)?;
    println!();
    
    // Generate features - only include features that are declared in Cargo.toml
    let features = generate_features(&config);
    let declared_features = collect_all_configs(&workspace);
    
    // Filter to only features that are actually declared in Cargo.toml
    let filtered_features: Vec<String> = features.into_iter()
        .filter(|f| declared_features.contains(f))
        .collect();
    
    println!("📋 Enabled features from .config:");
    for feature in &filtered_features {
        println!("  - {}", feature);
    }
    if filtered_features.is_empty() {
        println!("  (none - all CONFIG_* used via cfg flags)");
    }
    println!();
    
    // Build cargo command
    let mut cargo_args = vec!["build".to_string()];
    
    if !filtered_features.is_empty() {
        cargo_args.push("--features".to_string());
        cargo_args.push(filtered_features.join(","));
    }
    
    println!("🚀 Running: cargo {}\n", cargo_args.join(" "));
    
    // Set RUSTFLAGS to enable CONFIG_* as cfg values and declare them for check-cfg
    let mut rustflags = String::new();
    
    // Add check-cfg declarations for all CONFIG_* options from .config
    for config_name in all_configs.iter() {
        if !rustflags.is_empty() {
            rustflags.push(' ');
        }
        rustflags.push_str(&format!("--check-cfg=cfg({})", config_name));
    }
    
    // Add --cfg flags for ALL enabled configs from .config (not just features)
    for (key, value) in &config {
        if key.starts_with("CONFIG_") && (value == "y" || value == "m") {
            if !rustflags.is_empty() {
                rustflags.push(' ');
            }
            rustflags.push_str(&format!("--cfg {}", key));
        }
    }
    
    let mut cmd = process::Command::new("cargo");
    cmd.args(&cargo_args);
    cmd.current_dir(workspace_root);
    
    if !rustflags.is_empty() {
        cmd.env("RUSTFLAGS", rustflags);
    }
    
    let status = cmd.status()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;
    
    if !status.success() {
        return Err("Build failed".to_string());
    }
    
    println!("\n✅ Build completed successfully!");
    Ok(())
}





/// Print help message
fn print_help() {
    println!("cargo-kbuild - Kconfig-style build system for Rust");
    println!();
    println!("USAGE:");
    println!("    cargo-kbuild <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    build      Build project with current configuration");
    println!("    --help     Print this help message");
    println!("    --version  Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    cargo-kbuild build             # Build with .config");
    println!("    cargo-kbuild build --kconfig custom.config  # Use custom config file");
    println!();
    println!("NOTES:");
    println!("    - The .config file should be generated by external Kconfig tools");
    println!("      (e.g., Linux's 'make menuconfig')");
    println!("    - cargo-kbuild reads .config, generates config.rs, and builds your project");
    println!("    - CONFIG_* features are only needed for optional dependencies");
    println!();
}

/// Print version information
fn print_version() {
    println!("cargo-kbuild {}", env!("CARGO_PKG_VERSION"));
}

/// Build command - main build logic
fn cmd_build(command_args: &[String]) {
    // Find --kconfig argument
    let kconfig_path = command_args.iter()
        .position(|arg| arg == "--kconfig")
        .and_then(|i| command_args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or(".config");
    
    let workspace_root = std::env::current_dir()
        .expect("Failed to get current directory");
    let config_path = workspace_root.join(kconfig_path);
    
    if let Err(e) = build(&workspace_root, &config_path) {
        eprintln!("❌ Error: {}", e);
        process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // Handle cargo subcommand invocation (cargo-kbuild vs cargo kbuild)
    let command_args = if args.len() > 1 && args[1] == "kbuild" {
        &args[2..]
    } else if args.len() > 1 {
        &args[1..]
    } else {
        &[]
    };
    
    if command_args.is_empty() {
        eprintln!("Usage: cargo-kbuild <command>");
        eprintln!("Run 'cargo-kbuild --help' for more information");
        process::exit(1);
    }
    
    match command_args[0].as_str() {
        "build" => cmd_build(command_args),
        "--help" | "-h" | "help" => print_help(),
        "--version" | "-v" | "version" => print_version(),
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("Run 'cargo-kbuild --help' for available commands");
            process::exit(1);
        }
    }
}
