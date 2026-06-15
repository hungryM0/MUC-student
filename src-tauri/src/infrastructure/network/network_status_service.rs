use std::net::{IpAddr, UdpSocket};

use chrono::Local;

use crate::domain::models::NetworkStatus;
use crate::infrastructure::settings::AppSettings;

pub struct NetworkStatusService {
    settings: AppSettings,
}

impl NetworkStatusService {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    pub fn detect_network_status(&self) -> NetworkStatus {
        let ip = self.fetch_private_ipv4();
        let is_online = ip != "unknown" && !ip.is_empty();
        NetworkStatus {
            is_online,
            status_text: if is_online {
                "在线".to_string()
            } else {
                "未认证".to_string()
            },
            ip,
            checked_at: Local::now(),
        }
    }

    fn fetch_private_ipv4(&self) -> String {
        if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
            if sock.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = sock.local_addr() {
                    let ip = addr.ip().to_string();
                    if Self::is_private_ipv4(&ip) {
                        return ip;
                    }
                }
            }
        }

        let configured_ip = self.settings.preferred_source_ip.trim().to_string();
        if self.settings.bind_preferred_source_ip && Self::is_private_ipv4(&configured_ip) {
            return configured_ip;
        }
        "unknown".to_string()
    }

    fn is_private_ipv4(ip_text: &str) -> bool {
        ip_text
            .parse::<IpAddr>()
            .map(|addr| matches!(addr, IpAddr::V4(v4) if v4.is_private()))
            .unwrap_or(false)
    }
}
