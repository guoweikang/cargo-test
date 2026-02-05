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
    
    #[allow(dead_code)]
    fn find_crate(&self, name: &str) -> Option<&CrateInfo> {
        self.crates.iter().find(|c| c.name == name)
    }
}

/// Check if a dependency package supports kbuild
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

/// Collect all CONFIG_* feature names from workspace crates
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
    
    // Collect all CONFIG_* names and generate .cargo/config.toml
    let all_configs = collect_all_configs(&workspace);
    generate_cargo_config(workspace_root, &all_configs)?;
    println!();
    
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
    
    // Set RUSTFLAGS to enable CONFIG_* as cfg values and declare them for check-cfg
    let mut rustflags = String::new();
    
    // Add check-cfg declarations for all CONFIG_* options
    for config in all_configs.iter() {
        if !rustflags.is_empty() {
            rustflags.push(' ');
        }
        rustflags.push_str(&format!("--check-cfg=cfg({})", config));
    }
    
    // Add --cfg flags for enabled features
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

/// Initialize project configuration
fn cmd_init() {
    println!("🚀 Initializing cargo-kbuild configuration...\n");
    
    let workspace_root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("❌ Error: Failed to get current directory: {}", e);
            process::exit(1);
        }
    };
    
    // Parse workspace
    let workspace = match Workspace::new(workspace_root.clone()) {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            process::exit(1);
        }
    };
    
    // Collect all CONFIG_* features
    let all_configs = collect_all_configs(&workspace);
    
    if all_configs.is_empty() {
        println!("⚠️  No CONFIG_* features found in workspace");
        println!("   Please add CONFIG_* features to your crate's Cargo.toml");
        process::exit(0);
    }
    
    println!("📋 Found {} CONFIG_* features:", all_configs.len());
    let mut sorted_configs: Vec<_> = all_configs.iter().collect();
    sorted_configs.sort();
    for config in &sorted_configs {
        println!("   - {}", config);
    }
    println!();
    
    // Generate .config template
    let config_path = workspace_root.join(".config");
    if config_path.exists() {
        println!("ℹ️  .config file already exists, skipping template generation");
    } else {
        let mut content = String::from("# Kernel Configuration File\n");
        content.push_str("# Generated by cargo-kbuild init\n");
        content.push_str("# Edit this file to enable/disable features\n\n");
        
        for config in &sorted_configs {
            // Default to disabled
            content.push_str(&format!("# {}=y\n", config));
        }
        
        if let Err(e) = fs::write(&config_path, content) {
            eprintln!("❌ Error: Failed to write .config: {}", e);
            process::exit(1);
        }
        println!("✅ Created .config template");
    }
    
    // Update .gitignore
    let gitignore_path = workspace_root.join(".gitignore");
    let gitignore_content = fs::read_to_string(&gitignore_path).unwrap_or_default();
    
    let entries_to_add = vec![
        ".cargo/config.toml",
        "target/",
    ];
    
    let mut needs_update = false;
    let mut new_entries = Vec::new();
    
    for entry in &entries_to_add {
        if !gitignore_content.lines().any(|line| line.trim() == *entry) {
            needs_update = true;
            new_entries.push(*entry);
        }
    }
    
    if needs_update {
        let mut updated_content = gitignore_content;
        if !updated_content.is_empty() && !updated_content.ends_with('\n') {
            updated_content.push('\n');
        }
        updated_content.push_str("\n# cargo-kbuild generated files\n");
        for entry in new_entries {
            updated_content.push_str(entry);
            updated_content.push('\n');
        }
        
        if let Err(e) = fs::write(&gitignore_path, updated_content) {
            eprintln!("⚠️  Warning: Failed to update .gitignore: {}", e);
        } else {
            println!("✅ Updated .gitignore");
        }
    }
    
    println!();
    println!("✨ Initialization complete!");
    println!();
    println!("Next steps:");
    println!("1. Edit .config file to enable features (change # CONFIG_X=y to CONFIG_X=y)");
    println!("2. Run 'cargo-kbuild build' to compile your project");
    println!();
}

/// Check configuration validity
fn cmd_check() {
    println!("🔍 Checking configuration...\n");
    
    let workspace_root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("❌ Error: Failed to get current directory: {}", e);
            process::exit(1);
        }
    };
    
    let config_path = workspace_root.join(".config");
    
    // Check if .config exists
    if !config_path.exists() {
        eprintln!("❌ Error: .config file not found");
        eprintln!("   Run 'cargo-kbuild init' to create a template");
        process::exit(1);
    }
    
    // Parse workspace
    let workspace = match Workspace::new(workspace_root.clone()) {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            process::exit(1);
        }
    };
    
    // Parse .config
    let config = match parse_config(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            process::exit(1);
        }
    };
    
    println!("✓ .config syntax is valid");
    
    // Validate features
    if let Err(e) = validate_features(&workspace) {
        eprintln!("{}", e);
        process::exit(1);
    }
    
    // Collect all defined CONFIG_* in workspace
    let all_configs = collect_all_configs(&workspace);
    
    // Check for unused configs in .config
    let mut unused_configs = Vec::new();
    for (key, _value) in &config {
        if key.starts_with("CONFIG_") && !all_configs.contains(key) {
            unused_configs.push(key.clone());
        }
    }
    
    if !unused_configs.is_empty() {
        println!();
        println!("⚠️  Warning: The following configs are defined in .config but not declared in any crate:");
        for config in &unused_configs {
            println!("   - {}", config);
        }
        println!();
        println!("ℹ️  Suggestion: Remove them from .config or declare them as features in a crate's Cargo.toml");
    }
    
    // Check for undefined configs
    let mut undefined_configs = Vec::new();
    for config_name in &all_configs {
        if !config.contains_key(config_name) {
            undefined_configs.push(config_name);
        }
    }
    
    if !undefined_configs.is_empty() {
        println!();
        println!("ℹ️  Info: The following features are declared but not configured in .config:");
        let mut sorted: Vec<_> = undefined_configs.iter().collect();
        sorted.sort();
        for config in sorted {
            println!("   - {}", config);
        }
        println!();
        println!("ℹ️  These features will be disabled unless explicitly enabled in .config");
    }
    
    println!();
    println!("✅ Configuration check complete!");
}

/// Print help message
fn print_help() {
    println!("cargo-kbuild - Kconfig-style build system for Rust");
    println!();
    println!("USAGE:");
    println!("    cargo-kbuild <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    init       Initialize project configuration");
    println!("    check      Verify configuration and feature dependencies");
    println!("    build      Build project with current configuration");
    println!("    --help     Print this help message");
    println!("    --version  Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    cargo-kbuild init              # Create .config template");
    println!("    cargo-kbuild check             # Validate configuration");
    println!("    cargo-kbuild build             # Build with .config");
    println!("    cargo-kbuild build --kconfig custom.config  # Use custom config file");
    println!();
}

/// Print version information
fn print_version() {
    println!("cargo-kbuild {}", env!("CARGO_PKG_VERSION"));
}

/// Build command - main build logic
fn cmd_build(args: &[String]) {
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
        "init" => cmd_init(),
        "check" => cmd_check(),
        "build" => cmd_build(&args),
        "--help" | "-h" | "help" => print_help(),
        "--version" | "-v" | "version" => print_version(),
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("Run 'cargo-kbuild --help' for available commands");
            process::exit(1);
        }
    }
}
