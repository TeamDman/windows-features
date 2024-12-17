use clap::Arg;
use clap::Command;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use directories::ProjectDirs;
use itertools::Itertools;
use serde::Deserialize;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use tokio::process::Command as TokioCommand;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

/// The structure of the features.json based on the sample
#[derive(Debug, Deserialize)]
struct FeaturesFile {
    namespace_map: Vec<String>,
    feature_map: Vec<String>,
    namespaces: BTreeMap<String, Vec<NamespaceEntry>>,
}

#[derive(Debug, Deserialize)]
struct NamespaceEntry {
    name: String,
    features: Option<Vec<usize>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = Command::new("windows-features")
        .version("0.1.0")
        .about("Determines required features for windows-rs imports.")
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Enable debug output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("scan_dir")
                .long("scan-dir")
                .value_name("DIR")
                .help("Directory to scan for .rs files")
                .default_value("."),
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .help("Suppress all output except the final list of features")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let debug_enabled = matches.get_flag("debug");
    let quiet = matches.get_flag("quiet");
    let scan_dir = matches
        .get_one::<String>("scan_dir")
        .map(|s| s.to_string())
        .unwrap();

    // Setup tracing
    {
        let filter = if debug_enabled {
            EnvFilter::new("debug")
        } else if quiet {
            EnvFilter::new("warn")
        } else {
            EnvFilter::new("info")
        };

        tracing_subscriber::registry()
            .with(fmt::layer().without_time())
            .with(filter)
            .init();
    }

    info!("Starting windows-features tool");
    debug!("Debug mode enabled: {}", debug_enabled);
    debug!("Quiet mode: {}", quiet);
    debug!("Scan directory: {}", scan_dir);

    let project_dirs = ProjectDirs::from("ca", "teamdman", "windows-features")
        .ok_or_else(|| eyre!("Could not determine project directories"))?;
    let data_dir = project_dirs.data_dir();
    tokio::fs::create_dir_all(data_dir)
        .await
        .wrap_err("Failed to create data directory")?;

    let features_file = data_dir.join("features.json");
    let features = load_or_download_features_file(&features_file).await?;
    debug!(
        "Loaded features.json with {} namespaces",
        features.namespace_map.len()
    );

    // Build a map of namespace to required features
    let namespace_to_features = build_namespace_to_features(&features)?;

    // Run ripgrep to find windows imports
    let imports = find_imports(&scan_dir).await?;
    if imports.is_empty() {
        warn!("No 'use windows::' imports found.");
    }

    let mut required_features = BTreeSet::new();
    for import in &imports {
        debug!("Processing import: {}", import);
        if let Some(ns) = get_full_namespace(import) {
            debug!("  -> Reconstructed Namespace: {}", ns);
            if let Some(feats) = namespace_to_features.get(&ns) {
                for f in feats {
                    required_features.insert(f.clone());
                }
            } else {
                // Attempt to handle the LLM hallucination scenario:
                // The code tries to guess correct namespace by the last item name.
                if let Some(correct_feats) =
                    attempt_fix_import(&features, &namespace_to_features, import)
                {
                    for f in correct_feats {
                        required_features.insert(f);
                    }
                } else {
                    warn!(
                        "No features found for namespace: {} (import: {})",
                        ns, import
                    );
                }
            }
        } else {
            warn!("Could not determine namespace for import: {}", import);
        }
    }

    // Print required features
    if !quiet {
        eprintln!("Required windows-rs features:");
    }
    for f in &required_features {
        println!("{}", f);
    }

    Ok(())
}

/// Downloads or loads the features.json file
async fn load_or_download_features_file(path: &Path) -> Result<FeaturesFile> {
    if path.exists() {
        info!("features.json already exists locally at {}", path.display());
        let data = tokio::fs::read_to_string(path).await?;
        let parsed: FeaturesFile = serde_json::from_str(&data)?;
        return Ok(parsed);
    }

    let url = "https://raw.githubusercontent.com/microsoft/windows-rs/0.58.0/crates/libs/windows/features.json";
    info!("Downloading features.json from {}", url);
    let resp = reqwest::get(url).await?.text().await?;
    let parsed: FeaturesFile = serde_json::from_str(&resp)?;
    tokio::fs::write(path, resp).await?;
    Ok(parsed)
}

/// Builds a mapping from "Windows.X.Y.Z" namespaces to a set of features
fn build_namespace_to_features(
    features: &FeaturesFile,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut map = BTreeMap::new();
    for (idx_str, entries) in &features.namespaces {
        let idx: usize = idx_str.parse()?;
        if idx < features.namespace_map.len() {
            let namespace = &features.namespace_map[idx];
            let mut feature_set = BTreeSet::new();
            for e in entries {
                if let Some(feature_indexes) = &e.features {
                    for fi in feature_indexes {
                        if let Some(feature_name) = features.feature_map.get(*fi) {
                            feature_set.insert(feature_name.clone());
                        }
                    }
                }
            }
            map.insert(namespace.clone(), feature_set);
        } else {
            warn!("Index {} out of range for namespace_map", idx);
        }
    }

    Ok(map)
}

/// Runs ripgrep to find imports and returns a Vec of lines matching `use windows::`
async fn find_imports(scan_dir: &str) -> Result<Vec<String>> {
    // rg "use windows::" --type rust --no-heading --no-line-number
    let output = TokioCommand::new("rg")
        .arg("use windows::")
        .arg("--type")
        .arg("rust")
        .arg("--no-heading")
        .arg("--no-line-number")
        .arg(scan_dir)
        .output()
        .await?;

    if !output.status.success() && !output.stdout.is_empty() {
        error!("rg command returned non-zero exit code");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout
        .lines()
        .map(|l| l.trim().trim_end_matches(';').to_string())
        .unique()
        .collect();
    Ok(lines)
}

/// Given an import line like `use windows::Win32::Graphics::Gdi::DisplayConfigGetDeviceInfo;`
/// reconstruct the namespace: `Windows.Win32.Graphics.Gdi`
fn get_full_namespace(import_line: &str) -> Option<String> {
    let line = import_line.trim_end_matches(';').trim();
    let parts: Vec<&str> = line.split("use ").collect();
    let line = if parts.len() == 2 { parts[1] } else { line };
    let parts: Vec<&str> = line.split("::").collect();
    if parts.len() < 3 {
        return None;
    }

    // Ex: use windows::Win32::Graphics::Gdi::DisplayConfigGetDeviceInfo
    // parts = ["windows", "Win32", "Graphics", "Gdi", "DisplayConfigGetDeviceInfo"]
    // Reconstruct: Windows.Win32.Graphics.Gdi
    // Ex: If last part is '*', that means wildcard: Windows.Win32.Graphics.Gdi
    let last = parts.last().unwrap();
    let namespace_parts = if *last == "*" {
        &parts[1..parts.len() - 1]
    } else {
        &parts[1..parts.len() - 1]
    };

    let ns = format!("Windows.{}", namespace_parts.join("."));
    Some(ns)
}

/// Attempt to fix a mis-namespaced import by searching for the item name in all namespaces
/// Example: The code expects `use windows::Win32::Devices::Display::DisplayConfigGetDeviceInfo;`
/// But got `use windows::Win32::Graphics::Gdi::DisplayConfigGetDeviceInfo;`
fn attempt_fix_import(
    features: &FeaturesFile,
    namespace_map: &BTreeMap<String, BTreeSet<String>>,
    import: &str,
) -> Option<BTreeSet<String>> {
    // Extract the item name from the import (the last segment)
    let import = import.trim_end_matches(';').trim();
    let parts: Vec<&str> = import.split("::").collect();
    if parts.len() < 2 {
        return None;
    }
    let item_name = *parts.last().unwrap();

    // Search all namespaces for a match
    for (idx_str, entries) in &features.namespaces {
        for e in entries {
            if e.name == item_name {
                // Found a match for this item
                let idx: usize = idx_str.parse().ok()?;
                let namespace = features.namespace_map.get(idx)?;
                if let Some(feats) = namespace_map.get(namespace) {
                    warn!("Corrected namespace for {}: {}", item_name, namespace);
                    return Some(feats.clone());
                }
            }
        }
    }

    None
}
