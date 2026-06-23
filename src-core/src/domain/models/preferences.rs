use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub minimize_to_tray_on_close: bool,
    pub launch_on_startup: bool,
    pub auto_switch_account_on_traffic_exhausted: bool,
}
