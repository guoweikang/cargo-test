use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments
    let (kconfig_path, cargo_args) = parse_arguments(&args);
    
    println!("🔧 Cargo-Kbuild Wrapper");
    println!("📄 Config file: {}", kconfig_path.display());
    
    // Read and parse .config file
    let config = match read_config(&kconfig_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ Error reading config file: {}", e);
            exit(1);
        }
    };
    
    println!("✅ Loaded {} configuration options", config.len());
    
    // Scan workspace crates
    let workspace_root = find_workspace_root();
    let crates = match scan_workspace_crates(&workspace_root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Error scanning workspace: {}", e);
            exit(1);
        }
    };
    
    println!("📦 Found {} workspace crates", crates.len());
    
    // Validate features (check for sub-feature specifications)
    for crate_info in &crates {
        if let Err(e) = validate_features(&crate_info.features) {
            eprintln!("❌ Validation error in crate '{}': {}", crate_info.name, e);
            exit(1);
        }
    }
    
    println!("✅ Feature validation passed");
    
    // Generate RUSTFLAGS
    let rustflags = generate_rustflags(&config);
    
    // Generate features list
    let features = generate_features_list(&config);
    
    println!("🔨 RUSTFLAGS: {}", rustflags);
    if !features.is_empty() {
        println!("🎯 Features: {}", features);
    }
    
    // Call cargo with generated parameters
    let status = invoke_cargo(&rustflags, &features, &cargo_args);
    
    exit(status);
}

fn parse_arguments(args: &[String]) -> (PathBuf, Vec<String>) {
    let mut kconfig_path = PathBuf::from(".config");
    let mut cargo_args = Vec::new();
    let mut skip_next = false;
    
    for (i, arg) in args.iter().enumerate().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }
        
        if arg == "--kconfig" {
            if i + 1 < args.len() {
                kconfig_path = PathBuf::from(&args[i + 1]);
                skip_next = true;
            } else {
                eprintln!("❌ --kconfig requires a path argument");
                exit(1);
            }
        } else {
            cargo_args.push(arg.clone());
        }
    }
    
    (kconfig_path, cargo_args)
}

fn read_config(path: &Path) -> Result<HashMap<String, String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let mut config = HashMap::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse CONFIG_XXX=value format
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            
            // Only include enabled options (=y) or options with values
            if value == "y" || (!value.is_empty() && value != "n") {
                config.insert(key, value);
            }
        }
    }
    
    Ok(config)
}

fn find_workspace_root() -> PathBuf {
    env::current_dir().expect("Failed to get current directory")
}

#[derive(Debug)]
struct CrateInfo {
    name: String,
    path: PathBuf,
    has_kbuild_metadata: bool,
    features: HashMap<String, Vec<String>>,
}

fn scan_workspace_crates(workspace_root: &Path) -> Result<Vec<CrateInfo>, String> {
    let workspace_toml_path = workspace_root.join("Cargo.toml");
    let workspace_toml_content = fs::read_to_string(&workspace_toml_path)
        .map_err(|e| format!("Failed to read workspace Cargo.toml: {}", e))?;
    
    let workspace_toml: toml::Value = toml::from_str(&workspace_toml_content)
        .map_err(|e| format!("Failed to parse workspace Cargo.toml: {}", e))?;
    
    let mut crates = Vec::new();
    
    // Get workspace members
    if let Some(workspace) = workspace_toml.get("workspace") {
        if let Some(members) = workspace.get("members") {
            if let Some(members_array) = members.as_array() {
                for member in members_array {
                    if let Some(member_path) = member.as_str() {
                        let crate_path = workspace_root.join(member_path);
                        let cargo_toml_path = crate_path.join("Cargo.toml");
                        
                        if cargo_toml_path.exists() {
                            if let Ok(crate_info) = parse_crate_toml(&cargo_toml_path) {
                                crates.push(crate_info);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Also scan the root package if it exists
    if let Some(package) = workspace_toml.get("package") {
        if let Some(name) = package.get("name").and_then(|n| n.as_str()) {
            let features = extract_features(&workspace_toml);
            let has_kbuild_metadata = check_kbuild_metadata(&workspace_toml);
            
            crates.push(CrateInfo {
                name: name.to_string(),
                path: workspace_root.to_path_buf(),
                has_kbuild_metadata,
                features,
            });
        }
    }
    
    Ok(crates)
}

fn parse_crate_toml(path: &Path) -> Result<CrateInfo, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    
    let toml: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
    
    let name = toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| format!("Missing package name in {}", path.display()))?
        .to_string();
    
    let has_kbuild_metadata = check_kbuild_metadata(&toml);
    let features = extract_features(&toml);
    
    Ok(CrateInfo {
        name,
        path: path.parent().unwrap().to_path_buf(),
        has_kbuild_metadata,
        features,
    })
}

fn check_kbuild_metadata(toml: &toml::Value) -> bool {
    toml.get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("kbuild"))
        .and_then(|k| k.get("enabled"))
        .and_then(|e| e.as_bool())
        .unwrap_or(false)
}

fn extract_features(toml: &toml::Value) -> HashMap<String, Vec<String>> {
    let mut features = HashMap::new();
    
    if let Some(features_table) = toml.get("features") {
        if let Some(table) = features_table.as_table() {
            for (key, value) in table {
                if key.starts_with("CONFIG_") {
                    let deps: Vec<String> = if let Some(array) = value.as_array() {
                        array.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    } else {
                        Vec::new()
                    };
                    features.insert(key.clone(), deps);
                }
            }
        }
    }
    
    features
}

fn validate_features(features: &HashMap<String, Vec<String>>) -> Result<(), String> {
    for (feature_name, dependencies) in features {
        for dep in dependencies {
            // Check if dependency specifies a sub-feature (contains '/')
            if dep.contains('/') {
                return Err(format!(
                    "Feature '{}' specifies sub-feature: '{}'\n\
                    When using kbuild, dependencies should not specify sub-features.\n\
                    Expected: '{}'\n\
                    Reason: Dependency features are controlled by global config.",
                    feature_name,
                    dep,
                    dep.split('/').next().unwrap_or(dep)
                ));
            }
        }
    }
    Ok(())
}

fn generate_rustflags(config: &HashMap<String, String>) -> String {
    let cfg_flags: Vec<String> = config.keys()
        .map(|key| format!("--cfg {}", key))
        .collect();
    
    cfg_flags.join(" ")
}

fn generate_features_list(config: &HashMap<String, String>) -> String {
    let features: Vec<String> = config.keys().cloned().collect();
    features.join(",")
}

fn invoke_cargo(rustflags: &str, features: &str, cargo_args: &[String]) -> i32 {
    let mut cmd = Command::new("cargo");
    
    // Add cargo arguments
    for arg in cargo_args {
        cmd.arg(arg);
    }
    
    // Add features if any
    if !features.is_empty() {
        cmd.arg("--features");
        cmd.arg(features);
    }
    
    // Set RUSTFLAGS environment variable
    let existing_rustflags = env::var("RUSTFLAGS").unwrap_or_default();
    let combined_rustflags = if existing_rustflags.is_empty() {
        rustflags.to_string()
    } else {
        format!("{} {}", existing_rustflags, rustflags)
    };
    
    cmd.env("RUSTFLAGS", combined_rustflags);
    
    println!("\n🚀 Executing cargo...\n");
    
    // Execute cargo
    match cmd.status() {
        Ok(status) => {
            if status.success() {
                0
            } else {
                status.code().unwrap_or(1)
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to execute cargo: {}", e);
            1
        }
    }
}
