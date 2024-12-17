# Windows Features

The windows-rs crate uses feature flags to gate its functionality.

When leveraging Large Language Models (LLMs) or other automation tools to generate code, one often end up importing items without having the correct feature flags enabled.

This results in:

- Compilation failures due to missing features.
- Inclusion of unnecessary features, increasing build time.

This project solves these problems by scanning your code for `use windows::` imports and identifying the minimal set of feature flags required for your project.

This project expects the project to be scanned to be using the following `rustfmt.toml`, which causes imports to be placed on separate lines:

```toml
imports_granularity = "Item"
```

## Dependencies

- Ripgrep available in PATH as `rg`

## Usage

```pwsh
❯ cargo run -- --scan-dir D:\Repos\rust\monitor-scaling
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
     Running `target\debug\windows-features.exe --scan-dir D:\Repos\rust\monitor-scaling`
 INFO windows_features: Starting windows-features tool
 INFO windows_features: features.json already exists locally at C:\Users\TeamD\AppData\Roaming\teamdman\windows-features\data\features.json
Required windows-rs features:
Win32_Devices_Display
Win32_Foundation
Win32_Graphics_Gdi
```

```pwsh
❯ cargo run -- --help
   Compiling windows-features v0.1.0 (D:\Repos\rust\windows-features)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.96s
     Running `target\debug\windows-features.exe --help`
Determines required features for windows-rs imports.

Usage: windows-features.exe [OPTIONS]

Options:
      --debug           Enable debug output
      --scan-dir <DIR>  Directory to scan for .rs files [default: .]
      --quiet           Suppress all output except the final list of features
  -h, --help            Print help
  -V, --version         Print version
```

```pwsh
❯ cargo run -- --scan-dir D:\Repos\rust\monitor-scaling --debug
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.32s
     Running `target\debug\windows-features.exe --scan-dir D:\Repos\rust\monitor-scaling --debug`
 INFO windows_features: Starting windows-features tool
DEBUG windows_features: Debug mode enabled: true
DEBUG windows_features: Quiet mode: false
DEBUG windows_features: Scan directory: D:\Repos\rust\monitor-scaling
 INFO windows_features: features.json already exists locally at C:\Users\TeamD\AppData\Roaming\teamdman\windows-features\data\features.json
DEBUG windows_features: Loaded features.json with 680 namespaces
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DisplayConfigGetDeviceInfo;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DisplayConfigGetDeviceInfo
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DisplayConfigSetDeviceInfo;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DisplayConfigSetDeviceInfo
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::GetDisplayConfigBufferSizes;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: GetDisplayConfigBufferSizes
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::QueryDisplayConfig;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: QueryDisplayConfig
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_HEADER;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_DEVICE_INFO_HEADER
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_TYPE;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_DEVICE_INFO_TYPE
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_MODE_INFO
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_PATH_INFO
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DISPLAYCONFIG_SOURCE_MODE;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_SOURCE_MODE
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::DISPLAYCONFIG_TARGET_DEVICE_NAME;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_TARGET_DEVICE_NAME
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display", "Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::QDC_ONLY_ACTIVE_PATHS;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: QDC_ONLY_ACTIVE_PATHS
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Devices::Display::QUERY_DISPLAY_CONFIG_FLAGS;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Devices.Display
DEBUG windows_features:   -> Imported Item: QUERY_DISPLAY_CONFIG_FLAGS
DEBUG windows_features:      Found features for item: {"Win32_Devices_Display"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Foundation::ERROR_SUCCESS;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Foundation
DEBUG windows_features:   -> Imported Item: ERROR_SUCCESS
DEBUG windows_features:      Found features for item: {"Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Foundation::LUID;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Foundation
DEBUG windows_features:   -> Imported Item: LUID
DEBUG windows_features:      Found features for item: {"Win32_Foundation"}
DEBUG windows_features: Processing import: D:\Repos\rust\monitor-scaling\src\main.rs:use windows::Win32::Graphics::Gdi::DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
DEBUG windows_features:   -> Reconstructed Namespace: Windows.Win32.Graphics.Gdi
DEBUG windows_features:   -> Imported Item: DISPLAYCONFIG_PATH_MODE_IDX_INVALID
DEBUG windows_features:      Found features for item: {"Win32_Graphics_Gdi"}
Required windows-rs features:
Win32_Devices_Display
Win32_Foundation
Win32_Graphics_Gdi
```