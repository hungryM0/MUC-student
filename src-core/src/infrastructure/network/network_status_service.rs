use std::net::{IpAddr, ToSocketAddrs, UdpSocket};

use chrono::Local;

use crate::domain::models::NetworkStatus;
use crate::infrastructure::settings::AppSettings;

pub trait NetworkStatusDetector: Send + Sync {
    fn detect_network_status(&self) -> NetworkStatus;
}

pub struct NetworkStatusService {
    settings: AppSettings,
}

impl NetworkStatusService {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    fn fetch_private_ipv4(&self) -> String {
        if let Some((host, port)) = self.portal_route_target() {
            if let Ok(addresses) = (host.as_str(), port).to_socket_addrs() {
                for address in addresses.filter(|address| address.is_ipv4()) {
                    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
                        if sock.connect(address).is_ok() {
                            if let Ok(local_addr) = sock.local_addr() {
                                let ip = local_addr.ip().to_string();
                                if Self::is_private_ipv4(&ip) {
                                    return ip;
                                }
                            }
                        }
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

    fn portal_route_target(&self) -> Option<(String, u16)> {
        let url = url::Url::parse(self.settings.portal_url.trim()).ok()?;
        let host = url.host_str()?.to_string();
        let port = url.port_or_known_default()?;
        Some((host, port))
    }

    fn is_private_ipv4(ip_text: &str) -> bool {
        ip_text
            .parse::<IpAddr>()
            .map(|addr| matches!(addr, IpAddr::V4(v4) if v4.is_private()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_target_uses_portal_host_and_default_port() {
        let mut settings = AppSettings::default();
        settings.portal_url = "http://portal.example.test/srun_portal_pc.php".to_string();
        let service = NetworkStatusService::new(settings);

        assert_eq!(
            service.portal_route_target(),
            Some(("portal.example.test".to_string(), 80))
        );
    }

    #[test]
    fn route_target_preserves_explicit_port() {
        let mut settings = AppSettings::default();
        settings.portal_url = "https://portal.example.test:8443/login".to_string();
        let service = NetworkStatusService::new(settings);

        assert_eq!(
            service.portal_route_target(),
            Some(("portal.example.test".to_string(), 8443))
        );
    }
}

impl NetworkStatusDetector for NetworkStatusService {
    fn detect_network_status(&self) -> NetworkStatus {
        let ip = self.fetch_private_ipv4();
        let is_online = ip != "unknown" && !ip.is_empty();
        NetworkStatus {
            is_online,
            status_text: if is_online {
                "IP 已识别".to_string()
            } else {
                "IP 未识别".to_string()
            },
            ip,
            checked_at: Local::now(),
        }
    }
}
