use crate::application::error::AppResult;
use crate::infrastructure::network::http_transport::{HttpRequestSpec, HttpTransport};
use crate::infrastructure::parsers::legacy_portal_online_info_parser::{
    parse_legacy_portal_online_info, LegacyPortalOnlineInfo,
};
use crate::infrastructure::parsers::legacy_portal_success_page_parser::{
    parse_legacy_portal_success_page, LegacyPortalSuccessInfo,
};
use crate::infrastructure::parsers::portal_page_parser::join_url;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct LegacyPortalStatusClient {
    settings: AppSettings,
    transport: HttpTransport,
}

impl LegacyPortalStatusClient {
    pub fn new(settings: AppSettings, transport: HttpTransport) -> Self {
        Self {
            settings,
            transport,
        }
    }

    pub async fn fetch_online_info(&self) -> AppResult<LegacyPortalOnlineInfo> {
        let url = join_url(
            &self.settings.portal_url,
            "/include/auth_action.php?action=get_online_info&ajax=1",
        );
        let response = self
            .transport
            .request(HttpRequestSpec::get(url).max_redirects(1))
            .await?;
        parse_legacy_portal_online_info(&response.text)
    }

    pub async fn fetch_success_info(&self) -> AppResult<LegacyPortalSuccessInfo> {
        let url = join_url(&self.settings.portal_url, "/srun_portal_pc_success.php");
        let response = self
            .transport
            .request(HttpRequestSpec::get(url).max_redirects(1))
            .await?;
        parse_legacy_portal_success_page(&response.text)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::LegacyPortalStatusClient;
    use crate::application::error::AppError;
    use crate::infrastructure::network::http_transport::HttpTransport;
    use crate::infrastructure::settings::AppSettings;

    fn client(server: &MockServer) -> LegacyPortalStatusClient {
        let settings = AppSettings {
            portal_url: format!("{}/srun_portal_pc.php?ac_id=1&", server.uri()),
            ..Default::default()
        };
        LegacyPortalStatusClient::new(
            settings.clone(),
            HttpTransport::new(settings).expect("create transport"),
        )
    }

    #[tokio::test]
    async fn fetches_online_info_from_auth_action_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/include/auth_action.php"))
            .and(query_param("action", "get_online_info"))
            .and(query_param("ajax", "1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("41675360258,316056,0.00,c4:0f:08:8e:0e:5e,0,10.151.119.57"),
            )
            .mount(&server)
            .await;

        let info = client(&server)
            .fetch_online_info()
            .await
            .expect("fetch online info");

        assert_eq!(info.ip, "10.151.119.57");
        assert_eq!(info.used_traffic_bytes, "41675360258");
    }

    #[tokio::test]
    async fn fetches_success_info_from_success_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/srun_portal_pc_success.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "当前的ip：10.151.119.57\n上网用户：20260001\n已用流量：1.00G\n计费方式：flow",
            ))
            .mount(&server)
            .await;

        let info = client(&server)
            .fetch_success_info()
            .await
            .expect("fetch success info");

        assert_eq!(info.ip, "10.151.119.57");
        assert_eq!(info.username, "20260001");
        assert_eq!(info.used_traffic, "1.00G");
    }

    #[tokio::test]
    async fn maps_not_online_online_info_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/include/auth_action.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not_online"))
            .mount(&server)
            .await;

        let err = client(&server)
            .fetch_online_info()
            .await
            .expect_err("not online should fail");

        assert!(matches!(err, AppError::NotFound(_)));
    }
}
