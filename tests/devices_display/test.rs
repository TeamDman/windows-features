use windows::Win32::Devices::Display::DisplayConfigGetDeviceInfo;
use windows::Win32::Devices::Display::DisplayConfigSetDeviceInfo;
use windows::Win32::Devices::Display::GetDisplayConfigBufferSizes;
use windows::Win32::Devices::Display::QueryDisplayConfig;
use windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
use windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_HEADER;
use windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_TYPE;
use windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO;
use windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE;
use windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO;
use windows::Win32::Devices::Display::DISPLAYCONFIG_SOURCE_MODE;
use windows::Win32::Devices::Display::DISPLAYCONFIG_TARGET_DEVICE_NAME;
use windows::Win32::Devices::Display::QDC_ONLY_ACTIVE_PATHS;
use windows::Win32::Devices::Display::QUERY_DISPLAY_CONFIG_FLAGS;