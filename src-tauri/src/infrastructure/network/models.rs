use std::collections::HashMap;

use crate::domain::models::PortalHiddenFields;

#[derive(Clone, Debug)]
pub struct PortalPageData {
    pub login_url: String,
    pub html: String,
    pub hidden_fields: PortalHiddenFields,
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

#[derive(Clone, Debug)]
pub struct YiiLoginFormData {
    pub csrf_name: String,
    pub csrf_value: String,
    pub captcha_url: String,
    pub action_url: String,
    pub captcha_sum_hint: Option<u32>,
}
