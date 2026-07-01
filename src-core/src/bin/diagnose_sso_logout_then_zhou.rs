use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use muc_student_core::application::error::{AppError, AppResult};
use muc_student_core::infrastructure::network::http_transport::{
    build_form_headers, HttpRequestSpec, HttpTransport,
};
use muc_student_core::infrastructure::network::legacy_portal_auth_client::LegacyPortalAuthClient;
use muc_student_core::infrastructure::network::legacy_portal_status_client::LegacyPortalStatusClient;
use muc_student_core::infrastructure::parsers::panel_home_parser::{
    extract_csrf_meta, parse_panel_home,
};
use muc_student_core::infrastructure::parsers::portal_page_parser::join_url;
use muc_student_core::infrastructure::persistence::account_repository::AccountRepository;
use muc_student_core::infrastructure::persistence::database::AppDatabase;
use muc_student_core::infrastructure::persistence::runtime_paths::resolve_default_paths;
use muc_student_core::infrastructure::security::credential_vault::{
    CredentialVault, SystemCredentialVault,
};
use muc_student_core::infrastructure::settings::AppSettings;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("ERROR: {err}");
        std::process::exit(1);
    }
}

async fn run() -> AppResult<()> {
    let settings = AppSettings::default();
    let transport = HttpTransport::new(settings.clone())?;
    let status = LegacyPortalStatusClient::new(settings.clone(), transport.clone());
    let auth = LegacyPortalAuthClient::new(settings.clone(), transport.clone());

    let paths = resolve_default_paths()?;
    let db = AppDatabase::open(&paths)?;
    let vault: Arc<dyn CredentialVault> = Arc::new(SystemCredentialVault::initialize()?);
    let repo = AccountRepository::new(db, vault);
    let store = repo.load_store()?;
    let target_account = store
        .accounts
        .iter()
        .find(|account| account.username == "25011647" || account.remark_name.contains("周婧尧"))
        .ok_or_else(|| AppError::NotFound("找不到周婧尧（25011647）".to_string()))?;
    let target = repo.load_account_with_password(target_account)?;

    let before = status.fetch_success_info().await?;
    println!(
        "before username={} ip={} used={}",
        before.username, before.ip, before.used_traffic
    );

    let sso_response = transport
        .request(
            HttpRequestSpec::get(build_sso_url(
                &settings.traffic_portal_url,
                &before.username,
            ))
            .max_redirects(5)
            .preserve_redirect_cookies(),
        )
        .await?;
    let home_response = transport
        .request(
            HttpRequestSpec::get(join_url(&sso_response.final_url, "/home"))
                .cookies(sso_response.cookies.clone())
                .max_redirects(5)
                .preserve_redirect_cookies(),
        )
        .await?;
    let home = parse_panel_home(&home_response.text, Some(&before.ip))?;
    println!("panel_devices={}", home.online_devices.len());
    let device = home
        .matched_local_ip_device
        .ok_or_else(|| AppError::NotFound(format!("SSO 面板没找到本机 IP {}", before.ip)))?;
    println!(
        "matched_device ip={} id={} logout_path={}",
        device.ip, device.device_id, device.logout_path
    );

    let (csrf_param, csrf_token) = extract_csrf_meta(&home_response.text);
    let logout_url = join_url(&home_response.final_url, &decode_basic_html_entities(&device.logout_path));
    let mut headers = build_form_headers(&home_response.final_url);
    if !csrf_token.trim().is_empty() {
        headers.insert("X-CSRF-Token".to_string(), csrf_token.clone());
    }
    let mut payload = url::form_urlencoded::Serializer::new(String::new());
    if !csrf_param.trim().is_empty() && !csrf_token.trim().is_empty() {
        payload.append_pair(&csrf_param, &csrf_token);
    }
    let logout_response = transport
        .request(
            HttpRequestSpec::post(logout_url)
                .headers(headers)
                .body(payload.finish())
                .cookies(home_response.cookies.clone())
                .max_redirects(5)
                .preserve_redirect_cookies(),
        )
        .await?;
    println!(
        "logout status={} final={} body_head={}",
        logout_response.status,
        logout_response.final_url,
        compact_head(&logout_response.text)
    );

    for index in 0..8 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        match status.fetch_success_info().await {
            Ok(info) => println!(
                "after_logout_attempt={} still_online username={} ip={} used={}",
                index + 1,
                info.username,
                info.ip,
                info.used_traffic
            ),
            Err(err) => {
                println!("after_logout_attempt={} offline_or_error={err}", index + 1);
                break;
            }
        }
    }

    let result = auth.login_target_account(&target).await?;
    println!("login_success={}", result.success);
    println!("login_already_online={}", result.already_online);
    println!("login_message={}", result.message);
    println!("login_response={}", result.response_text);

    for index in 0..10 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        match status.fetch_success_info().await {
            Ok(info) => println!(
                "confirm_attempt={} matched={} username={} ip={} used={}",
                index + 1,
                info.username.trim() == target.account.username.trim(),
                info.username,
                info.ip,
                info.used_traffic
            ),
            Err(err) => println!("confirm_attempt={} error={err}", index + 1),
        }
    }

    Ok(())
}

fn build_sso_url(base_url: &str, username: &str) -> String {
    let clean_username = username.trim();
    let data = base64::engine::general_purpose::STANDARD
        .encode(format!("{clean_username}:{clean_username}").as_bytes());
    join_url(base_url, &format!("/site/sso?data={data}"))
}

fn compact_head(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(160)
        .collect()
}

fn decode_basic_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
