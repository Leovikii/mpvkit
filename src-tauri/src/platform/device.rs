use crate::device::{AdvancedColor, DeviceReport, Display, recommendation};
use windows_sys::Win32::{Devices::Display::*, Graphics::Gdi::*, System::Power::*};
fn text(buffer: &[u16]) -> String {
    String::from_utf16_lossy(&buffer[..buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len())])
}

pub fn detect_device() -> DeviceReport {
    let mut notes = vec![
        "高级色彩支持/开启不等于已验证 HDR 输出；HDR 类型、色彩链与媒体匹配仍需播放验证。".into(),
        "GPU 名称来自活动显示适配器；不包含未连接显示器的独显，解码支持以真实播放状态为准。".into(),
    ];
    let mut adapters = Vec::new();
    let mut displays = Vec::new();
    for index in 0..32 {
        let mut device = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        // SAFETY: correctly sized out struct and null root adapter name.
        if unsafe { EnumDisplayDevicesW(std::ptr::null(), index, &mut device, 0) } == 0 {
            break;
        }
        if device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP == 0
            || device.StateFlags & DISPLAY_DEVICE_MIRRORING_DRIVER != 0
        {
            continue;
        }
        let adapter = text(&device.DeviceString);
        if !adapters.contains(&adapter) {
            adapters.push(adapter.clone());
        }
        let mut mode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        if unsafe {
            EnumDisplaySettingsW(device.DeviceName.as_ptr(), ENUM_CURRENT_SETTINGS, &mut mode)
        } != 0
        {
            displays.push(Display {
                name: text(&device.DeviceName),
                id: text(&device.DeviceName),
                adapter: adapter.clone(),
                primary: device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE != 0,
                hdr_supported: None,
                hdr_enabled: None,
                width: mode.dmPelsWidth,
                height: mode.dmPelsHeight,
                refresh_hz: (mode.dmDisplayFrequency > 1).then_some(mode.dmDisplayFrequency),
            });
        } else {
            notes.push("部分显示模式不可查询，已保留为未知。".into());
        }
    }
    let mut power = SYSTEM_POWER_STATUS::default();
    let available = unsafe { GetSystemPowerStatus(&mut power) } != 0;
    let on_battery = if available {
        match power.ACLineStatus {
            0 => Some(true),
            1 => Some(false),
            _ => None,
        }
    } else {
        None
    };
    let battery_saver = available.then_some(power.SystemStatusFlag == 1);
    let battery_percent = (available
        && power.BatteryFlag != 255
        && power.BatteryFlag & 128 == 0
        && power.BatteryLifePercent <= 100)
        .then_some(power.BatteryLifePercent);
    if !available {
        notes.push("系统电源状态不可查询。".into());
    }
    let advanced_color = advanced_color(&mut notes);
    for display in &mut displays {
        let outputs: Vec<_> = advanced_color
            .iter()
            .filter(|c| c.source == display.name)
            .collect();
        // Mirrored targets must not borrow a different monitor's HDR state.
        if outputs.len() == 1 {
            display.id = outputs[0].id.clone();
            display.hdr_supported = outputs[0].hdr_supported;
            display.hdr_enabled = outputs[0].hdr_enabled;
        }
    }
    displays.sort_by_key(|d| !d.primary);
    let (recommended_profile, reason) = recommendation(on_battery, battery_saver);
    DeviceReport {
        preferred_display: displays.first().map(|d| d.id.clone()),
        adapters,
        displays,
        advanced_color,
        on_battery,
        battery_percent,
        battery_saver,
        recommended_profile,
        reason,
        notes,
    }
}

fn advanced_color(notes: &mut Vec<String>) -> Vec<AdvancedColor> {
    // A changing display topology can require a larger buffer; retry at most three times.
    for _ in 0..3 {
        let (mut path_count, mut mode_count) = (0, 0);
        let result = unsafe {
            GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
        };
        if result != 0 || path_count > 64 || mode_count > 256 {
            break;
        }
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
        let result = unsafe {
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                std::ptr::null_mut(),
            )
        };
        if result == 122 {
            continue;
        }
        if result != 0 {
            break;
        }
        let mut outputs = Vec::new();
        for path in &paths[..path_count as usize] {
            let target = path.targetInfo;
            let mut name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                    size: std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                    adapterId: target.adapterId,
                    id: target.id,
                },
                ..Default::default()
            };
            let name_ok = unsafe { DisplayConfigGetDeviceInfo(&mut name.header) } == 0;
            let mut color = DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO,
                    size: std::mem::size_of::<DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO>() as u32,
                    adapterId: target.adapterId,
                    id: target.id,
                },
                ..Default::default()
            };
            let ok = unsafe { DisplayConfigGetDeviceInfo(&mut color.header) } == 0;
            let bits = unsafe { color.Anonymous.value };
            let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                    size: std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                    adapterId: path.sourceInfo.adapterId,
                    id: path.sourceInfo.id,
                },
                ..Default::default()
            };
            let source_ok = unsafe { DisplayConfigGetDeviceInfo(&mut source.header) } == 0;
            // Windows SDK 10.0.26100 wingdi.h: this API distinguishes HDR from WCG.
            // Older systems may reject type 15; never infer HDR from legacy advanced-color bits.
            let mut hdr = AdvancedColorInfo2 {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: 15,
                    size: std::mem::size_of::<AdvancedColorInfo2>() as u32,
                    adapterId: target.adapterId,
                    id: target.id,
                },
                ..Default::default()
            };
            let hdr_ok = unsafe { DisplayConfigGetDeviceInfo(&mut hdr.header) } == 0;
            outputs.push(AdvancedColor {
                source: if source_ok {
                    text(&source.viewGdiDeviceName)
                } else {
                    String::new()
                },
                id: if name_ok {
                    text(&name.monitorDevicePath)
                } else {
                    format!(
                        "{}:{}:{}",
                        target.adapterId.HighPart, target.adapterId.LowPart, target.id
                    )
                },
                hdr_supported: hdr_ok.then_some(hdr.flags & 16 != 0),
                hdr_enabled: hdr_ok.then_some(hdr.active_mode == 2),
                name: if name_ok {
                    text(&name.monitorFriendlyDeviceName)
                } else {
                    "未命名显示目标".into()
                },
                supported: ok.then_some(bits & 1 != 0),
                enabled: ok.then_some(bits & 2 != 0),
                bits_per_channel: ok.then_some(color.bitsPerColorChannel),
            });
            if !ok {
                notes.push("部分输出不提供高级色彩信息（驱动、远程会话或系统能力限制）。".into());
            }
        }
        return outputs;
    }
    notes.push("活动输出拓扑/高级色彩不可查询；可能为远程会话、权限或驱动限制。".into());
    Vec::new()
}

#[repr(C)]
#[derive(Default)]
struct AdvancedColorInfo2 {
    header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    flags: u32,
    encoding: i32,
    bits_per_channel: u32,
    active_mode: i32,
}

pub fn detect_device_for_window(handle: isize) -> DeviceReport {
    let mut report = detect_device();
    if handle != 0 {
        let monitor = unsafe { MonitorFromWindow(handle as _, MONITOR_DEFAULTTOPRIMARY) };
        let mut info = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            ..Default::default()
        };
        if unsafe { GetMonitorInfoW(monitor, (&mut info as *mut MONITORINFOEXW).cast()) } != 0
            && let Some(display) = report
                .displays
                .iter()
                .find(|d| d.name == text(&info.szDevice))
        {
            report.preferred_display = Some(display.id.clone());
        }
    }
    report
}
