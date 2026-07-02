use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub minimize_to_tray_on_close: bool,
    pub launch_on_startup: bool,
    pub auto_switch_account_on_traffic_exhausted: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            minimize_to_tray_on_close: true,
            launch_on_startup: false,
            auto_switch_account_on_traffic_exhausted: false,
        }
    }
}
