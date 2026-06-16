use crate::application::error::{AppError, AppResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyPortalOnlineInfo {
    pub username: String,
    pub online_seconds: String,
    pub used_traffic: String,
    pub mac: String,
    pub billing_state: String,
    pub ip: String,
}

pub fn parse_legacy_portal_online_info(raw: &str) -> AppResult<LegacyPortalOnlineInfo> {
    let clean = raw.trim().trim_matches('\u{feff}').trim();
    if clean.is_empty() {
        return Err(AppError::Network("旧门户在线信息为空".to_string()));
    }
    if clean.contains("not_online") || clean.contains("not online") {
        return Err(AppError::NotFound("当前 IP 未在线".to_string()));
    }

    let fields = clean.split(',').map(str::trim).collect::<Vec<_>>();
    if fields.len() < 6 {
        return Err(AppError::Network(format!(
            "旧门户在线信息字段不足：{}",
            clean.chars().take(120).collect::<String>()
        )));
    }

    let username = fields[0].to_string();
    let ip = fields[5].to_string();
    if username.is_empty() || ip.is_empty() {
        return Err(AppError::Network(format!(
            "旧门户在线信息缺少账号或 IP：{}",
            clean.chars().take(120).collect::<String>()
        )));
    }

    Ok(LegacyPortalOnlineInfo {
        username,
        online_seconds: fields[1].to_string(),
        used_traffic: fields[2].to_string(),
        mac: fields[3].to_string(),
        billing_state: fields[4].to_string(),
        ip,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_online_info() {
        let info = parse_legacy_portal_online_info(
            "13377235977,316056,7.10,c4:0f:08:8e:0e:5e,0,10.151.119.57",
        )
        .unwrap();

        assert_eq!(info.username, "13377235977");
        assert_eq!(info.ip, "10.151.119.57");
        assert_eq!(info.mac, "c4:0f:08:8e:0e:5e");
    }

    #[test]
    fn rejects_incomplete_payload() {
        assert!(parse_legacy_portal_online_info("13377235977,316056").is_err());
    }
}
