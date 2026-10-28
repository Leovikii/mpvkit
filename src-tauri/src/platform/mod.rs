#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub mod windows_mpv;
#[cfg(target_os = "windows")]
pub use windows_mpv as core;
#[cfg(target_os = "linux")]
#[path = "linux_core.rs"]
pub mod core;
#[cfg(target_os = "windows")]
pub use windows::*;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "linux")]
#[path = "linux_app_update.rs"]
pub mod app_update;

#[cfg(target_os = "windows")]
pub mod appearance;
#[cfg(target_os = "linux")]
#[path = "linux_appearance.rs"]
pub mod appearance;
#[cfg(target_os = "windows")]
pub mod desktop;
#[cfg(target_os = "linux")]
#[path = "linux_desktop.rs"]
pub mod desktop;

#[cfg(target_os = "windows")]
pub const UOSC_HELPER: &str = "scripts/uosc/bin/ziggy-windows.exe";
#[cfg(target_os = "linux")]
pub const UOSC_HELPER: &str = "scripts/uosc/bin/ziggy-linux";

#[cfg(all(test, target_os = "windows"))]
#[path = "linux_core.rs"]
mod linux_core_contract;
