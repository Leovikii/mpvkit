use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum Language {
    #[serde(rename = "zh-Hans")]
    Chinese,
    #[default]
    #[serde(rename = "en")]
    English,
}

impl Language {
    pub fn render(self, template: &str) -> String {
        let texts: BTreeMap<String, [String; 2]> =
            serde_json::from_str(include_str!("../../../resources/menu-text.json"))
                .expect("embedded menu text");
        let mut result = template.to_string();
        for (key, pair) in texts {
            result = result.replace(
                &format!("__{key}__"),
                &pair[usize::from(self == Self::English)],
            );
        }
        result
    }

    pub fn uosc(self) -> &'static str {
        match self {
            Self::Chinese => "zh-hans,en",
            Self::English => "en",
        }
    }
}
