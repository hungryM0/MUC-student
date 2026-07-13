use serde::Deserialize;
use tauri::plugin::{Builder, PluginHandle, TauriPlugin};
use tauri::{AppHandle, Manager, Runtime};

const PLUGIN_IDENTIFIER: &str = "cn.muc.student";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidNetworkContext {
    pub bound: bool,
    pub ipv4: String,
    pub network_handle: String,
    pub detail: String,
}

struct AndroidNetwork<R: Runtime> {
    handle: PluginHandle<R>,
}

impl<R: Runtime> AndroidNetwork<R> {
    fn await_wifi_context(&self, timeout_ms: u64) -> Result<AndroidNetworkContext, String> {
        self.handle
            .run_mobile_plugin(
                "awaitWifiContext",
                serde_json::json!({ "timeoutMs": timeout_ms }),
            )
            .map_err(|err| err.to_string())
    }
}

pub fn await_wifi_context<R: Runtime>(
    app: &AppHandle<R>,
    timeout_ms: u64,
) -> Result<AndroidNetworkContext, String> {
    app.state::<AndroidNetwork<R>>()
        .await_wifi_context(timeout_ms)
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("android-network")
        .setup(|app, api| {
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "CampusNetworkPlugin")?;
            app.manage(AndroidNetwork { handle });
            Ok(())
        })
        .build()
}
