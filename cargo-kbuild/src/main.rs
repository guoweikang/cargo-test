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
    #[serde(default)]
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
    
    fn find_crate(&self, name: &str) -> Option<&CrateInfo> {
        self.crates.iter().find(|c| c.name == name)
    }
}

/// Check if a dependency package supports kbuild
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
    
    // Validate features
    validate_features(&workspace)?;
    
    // Parse .config
    let config = parse_config(config_path)?;
    
    // Generate config.rs file with constants
    generate_config_rs(workspace_root, &config)?;
    println!();
    
    // Generate features
    let features = generate_features(&config);
    
    println!("📋 Enabled features from .config:");
    for feature in &features {
        println!("  - {}", feature);
    }
    println!();
    
    // Build cargo command
    let mut cargo_args = vec!["build".to_string()];
    
    if !features.is_empty() {
        cargo_args.push("--features".to_string());
        cargo_args.push(features.join(","));
    }
    
    println!("🚀 Running: cargo {}\n", cargo_args.join(" "));
    
    // Set RUSTFLAGS to enable CONFIG_* as cfg values
    let mut rustflags = String::new();
    for feature in &features {
        if !rustflags.is_empty() {
            rustflags.push(' ');
        }
        rustflags.push_str(&format!("--cfg {}", feature));
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: cargo-kbuild <command> [options]");
        eprintln!("Commands:");
        eprintln!("  build --kconfig <path>  Build with kernel config");
        process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "build" => {
            // Find --kconfig argument
            let kconfig_path = args.iter()
                .position(|arg| arg == "--kconfig")
                .and_then(|i| args.get(i + 1))
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
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Available commands: build");
            process::exit(1);
        }
    }
}
