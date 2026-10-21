use crate::config::Profile;
use serde::Serialize;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Display {
    pub name: String,
    pub id: String,
    pub adapter: String,
    pub primary: bool,
    pub hdr_supported: Option<bool>,
    pub hdr_enabled: Option<bool>,
    pub width: u32,
    pub height: u32,
    pub refresh_hz: Option<u32>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedColor {
    pub source: String,
    pub id: String,
    pub hdr_supported: Option<bool>,
    pub hdr_enabled: Option<bool>,
    pub name: String,
    pub supported: Option<bool>,
    pub enabled: Option<bool>,
    pub bits_per_channel: Option<u32>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceReport {
    pub adapters: Vec<String>,
    pub displays: Vec<Display>,
    pub advanced_color: Vec<AdvancedColor>,
    pub on_battery: Option<bool>,
    pub battery_percent: Option<u8>,
    pub battery_saver: Option<bool>,
    pub recommended_profile: Profile,
    pub preferred_display: Option<String>,
    pub reason: &'static str,
    pub notes: Vec<String>,
}
pub fn recommendation(
    on_battery: Option<bool>,
    battery_saver: Option<bool>,
) -> (Profile, &'static str) {
    if on_battery == Some(true) || battery_saver == Some(true) {
        (
            Profile::PowerSaving,
            "系统报告电池供电或节电模式，建议先降低缩放处理成本；实际功耗收益未测。",
        )
    } else {
        (
            Profile::Balanced,
            "优先使用目标 MPV 的均衡缩放默认值；GPU 名称和分辨率不足以证明应选高画质。",
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_device_does_not_become_high_quality_and_power_hint_is_explicit() {
        assert_eq!(recommendation(None, None).0, Profile::Balanced);
        assert_eq!(
            recommendation(Some(false), Some(false)).0,
            Profile::Balanced
        );
        assert_eq!(recommendation(Some(true), None).0, Profile::PowerSaving);
        assert_eq!(recommendation(None, Some(true)).0, Profile::PowerSaving);
    }
}
