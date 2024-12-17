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
    // Initialize color-eyre for better error reports
    color_eyre::install()?;

    // Define and parse command-line arguments
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

    // Setup tracing for logging
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

    // Determine project directories for storing data
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

    // Build mappings:
    // 1. Namespace -> Features
    // 2. Item -> Features
    let (namespace_to_features, item_to_features) = build_feature_mappings(&features)?;

    // Run ripgrep to find windows imports
    let imports = find_imports(&scan_dir).await?;
    if imports.is_empty() {
        warn!("No 'use windows::' imports found.");
    }

    let mut required_features = BTreeSet::new();
    for import in &imports {
        debug!("Processing import: {}", import);
        // Extract file path and import line
        let (file_path, import_line) = parse_import_line(import)?;

        // Get the full namespace and item name
        if let Some((ns, item)) = parse_namespace_and_item(&import_line) {
            debug!("  -> Reconstructed Namespace: {}", ns);
            debug!("  -> Imported Item: {}", item);

            if let Some(features) = item_to_features.get(&item) {
                debug!("     Found features for item: {:?}", features);
                required_features.extend(features.clone());
            } else {
                // Attempt to handle the LLM hallucination scenario:
                // The code tries to guess the correct namespace by the item name.
                if let Some(correct_feats) = attempt_fix_import(&features, &item_to_features, &item)
                {
                    required_features.extend(correct_feats);
                } else {
                    warn!(
                        "No features found for item: {} (import: {})",
                        item, import_line
                    );
                }
            }
        } else {
            warn!(
                "Could not determine namespace and item for import: {}",
                import
            );
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
        let data = tokio::fs::read_to_string(path)
            .await
            .wrap_err("Failed to read features.json")?;
        let parsed: FeaturesFile =
            serde_json::from_str(&data).wrap_err("Failed to parse features.json")?;
        return Ok(parsed);
    }

    let url = "https://raw.githubusercontent.com/microsoft/windows-rs/0.58.0/crates/libs/windows/features.json";
    info!("Downloading features.json from {}", url);
    let resp = reqwest::get(url)
        .await
        .wrap_err("Failed to download features.json")?
        .text()
        .await
        .wrap_err("Failed to read response body")?;
    let parsed: FeaturesFile =
        serde_json::from_str(&resp).wrap_err("Failed to parse downloaded features.json")?;
    tokio::fs::write(path, &resp)
        .await
        .wrap_err("Failed to write features.json")?;
    Ok(parsed)
}

/// Builds mappings:
/// 1. Namespace -> Features
/// 2. Item -> Features
fn build_feature_mappings(
    features: &FeaturesFile,
) -> Result<(
    BTreeMap<String, BTreeSet<String>>,
    BTreeMap<String, BTreeSet<String>>,
)> {
    let mut namespace_to_features = BTreeMap::new();
    let mut item_to_features = BTreeMap::new();

    for (idx_str, entries) in &features.namespaces {
        let idx: usize = idx_str.parse().wrap_err("Invalid namespace index")?;
        if idx >= features.namespace_map.len() {
            warn!("Index {} out of range for namespace_map", idx);
            continue;
        }
        let namespace = &features.namespace_map[idx];

        let mut namespace_features = BTreeSet::new();
        for entry in entries {
            if let Some(feature_indexes) = &entry.features {
                for &fi in feature_indexes {
                    if let Some(feature_name) = features.feature_map.get(fi) {
                        namespace_features.insert(feature_name.clone());
                        // Also map the individual item to its features
                        item_to_features
                            .entry(entry.name.clone())
                            .or_insert_with(BTreeSet::new)
                            .insert(feature_name.clone());
                    } else {
                        warn!("Feature index {} out of bounds for feature_map", fi);
                    }
                }
            }
        }

        namespace_to_features.insert(namespace.clone(), namespace_features);
    }

    Ok((namespace_to_features, item_to_features))
}

/// Runs ripgrep to find imports and returns a Vec of lines matching `use windows::`
/// The expected format is `file_path:use windows::...;`
async fn find_imports(scan_dir: &str) -> Result<Vec<String>> {
    // rg "use windows::" --type rust --no-heading --no-line-number
    let output = TokioCommand::new("rg")
        .arg("use windows::")
        .arg("--type")
        .arg("rust")
        .arg("--no-heading")
        .arg("--no-line-number")
        .arg("--with-filename") // Include filenames in the output
        .arg(scan_dir)
        .output()
        .await
        .wrap_err("Failed to execute ripgrep (rg)")?;

    if !output.status.success() && !output.stdout.is_empty() {
        error!("rg command returned non-zero exit code");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout
        .lines()
        .map(|l| l.trim().to_string())
        .unique()
        .collect();
    Ok(lines)
}

/// Parses a line from ripgrep output into file path and import line.
/// Expected input format: `file_path:use windows::...;`
fn parse_import_line(line: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(eyre!("Invalid import line format: {}", line));
    }
    let file_path = parts[0].to_string();
    let import_line = parts[1].to_string();
    Ok((file_path, import_line))
}

/// Given an import line like `use windows::Win32::Graphics::Gdi::DisplayConfigGetDeviceInfo;`,
/// reconstruct the namespace and extract the item name.
fn parse_namespace_and_item(import_line: &str) -> Option<(String, String)> {
    let line = import_line.trim_end_matches(';').trim();
    let parts: Vec<&str> = line.split("::").collect();
    if parts.len() < 3 {
        return None;
    }

    // Ex: use windows::Win32::Graphics::Gdi::DisplayConfigGetDeviceInfo
    // parts = ["use windows", "Win32", "Graphics", "Gdi", "DisplayConfigGetDeviceInfo"]
    // Reconstruct namespace: Windows.Win32.Graphics.Gdi
    // Item: DisplayConfigGetDeviceInfo

    let namespace_parts = &parts[1..parts.len() - 1];
    let item = parts.last()?.to_string();
    let namespace = format!("Windows.{}", namespace_parts.join("."));

    Some((namespace, item))
}

/// Attempt to fix a mis-namespaced import by searching for the item name in all namespaces
fn attempt_fix_import(
    features: &FeaturesFile,
    item_to_features: &BTreeMap<String, BTreeSet<String>>,
    item_name: &str,
) -> Option<BTreeSet<String>> {
    // Search for the item name across all items
    for (existing_item, feats) in item_to_features {
        if existing_item.eq_ignore_ascii_case(item_name) {
            warn!(
                "Corrected namespace for {} to match item name: {}",
                item_name, existing_item
            );
            return Some(feats.clone());
        }
    }

    // As a fallback, perform a case-sensitive search
    for (existing_item, feats) in item_to_features {
        if existing_item == item_name {
            warn!(
                "Corrected namespace for {} to match item name: {}",
                item_name, existing_item
            );
            return Some(feats.clone());
        }
    }

    None
}
