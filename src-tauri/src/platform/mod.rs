mod windows;

pub use windows::*;

mod device;
pub use device::{detect_device, detect_device_for_window};

pub mod registry;

pub mod desktop;

pub mod appearance;
