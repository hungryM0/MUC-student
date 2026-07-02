use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub portal_url: String,
    pub traffic_portal_url: String,
    pub preferred_interface_name: String,
    pub preferred_source_ip: String,
    pub bind_preferred_source_ip: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            portal_url: "http://rz.muc.edu.cn/srun_portal_pc.php?ac_id=1&".to_string(),
            traffic_portal_url: "http://192.168.2.231:8800/home".to_string(),
            preferred_interface_name: "WLAN".to_string(),
            preferred_source_ip: String::new(),
            bind_preferred_source_ip: false,
        }
    }
}
