use std::collections::HashMap;

use crate::domain::models::PortalHiddenFields;

#[derive(Clone, Debug)]
pub struct PortalPageData {
    pub login_url: String,
    pub html: String,
    pub hidden_fields: PortalHiddenFields,
    pub cookies: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct HttpResponseData {
    pub final_url: String,
    pub status: u16,
    pub reason: String,
    pub raw_body: Vec<u8>,
    pub text: String,
    pub cookies: HashMap<String, String>,
}
