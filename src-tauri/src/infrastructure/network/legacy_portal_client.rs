use std::collections::HashMap;

use crate::application::error::AppResult;
use crate::infrastructure::network::http_transport::HttpTransport;
use crate::infrastructure::parsers::legacy_portal_online_info_parser::{
    parse_legacy_portal_online_info, LegacyPortalOnlineInfo,
};
use crate::infrastructure::parsers::legacy_portal_success_page_parser::{
    parse_legacy_portal_success_page, LegacyPortalSuccessInfo,
};
use crate::infrastructure::parsers::portal_page_parser::join_url;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct LegacyPortalClient {
    settings: AppSettings,
    transport: HttpTransport,
}

impl LegacyPortalClient {
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
            .request(
                "GET",
                &url,
                HashMap::new(),
                String::new(),
                HashMap::new(),
                1,
            )
            .await?;
        parse_legacy_portal_online_info(&response.text)
    }

    pub async fn fetch_success_info(&self) -> AppResult<LegacyPortalSuccessInfo> {
        let url = join_url(&self.settings.portal_url, "/srun_portal_pc_success.php");
        let response = self
            .transport
            .request(
                "GET",
                &url,
                HashMap::new(),
                String::new(),
                HashMap::new(),
                1,
            )
            .await?;
        parse_legacy_portal_success_page(&response.text)
    }
}
