use crate::application::error::{AppError, AppResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyPortalOnlineInfo {
    pub used_traffic_bytes: String,
    pub online_seconds: String,
    pub balance: String,
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

    let used_traffic_bytes = fields[0].to_string();
    let ip = fields[5].to_string();
    if used_traffic_bytes.is_empty() || ip.is_empty() {
        return Err(AppError::Network(format!(
            "旧门户在线信息缺少已用流量字节数或 IP：{}",
            clean.chars().take(120).collect::<String>()
        )));
    }

    Ok(LegacyPortalOnlineInfo {
        used_traffic_bytes,
        online_seconds: fields[1].to_string(),
        balance: fields[2].to_string(),
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
            "41675360258,316056,0.00,c4:0f:08:8e:0e:5e,0,10.151.119.57",
        )
        .unwrap();

        assert_eq!(info.used_traffic_bytes, "41675360258");
        assert_eq!(info.online_seconds, "316056");
        assert_eq!(info.balance, "0.00");
        assert_eq!(info.ip, "10.151.119.57");
        assert_eq!(info.mac, "c4:0f:08:8e:0e:5e");
    }

    #[test]
    fn rejects_incomplete_payload() {
        assert!(parse_legacy_portal_online_info("13377235977,316056").is_err());
    }

    #[test]
    fn treats_not_online_as_not_found() {
        assert!(matches!(
            parse_legacy_portal_online_info("not_online"),
            Err(AppError::NotFound(_))
        ));
    }
}
