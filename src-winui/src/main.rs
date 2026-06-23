#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod platform;
mod ui;

use std::sync::Arc;

use muc_student_core::application::{AppCore, NoopEventSink};
use platform::{RunKeyStartupController, Win32RuntimePathProvider};
use ui::WinuiApp;
use windows_core::Result;
use winui3::bootstrap::{PackageDependency, WindowsAppSDKVersion};
use winui3::Microsoft::UI::Xaml::{
    Application, ApplicationInitializationCallback, ApplicationInitializationCallbackParams,
};
use winui3::{init_apartment, ApartmentType};

fn main() -> Result<()> {
    force_chinese_locale();

    init_apartment(ApartmentType::SingleThreaded)?;
    let _dependency = PackageDependency::initialize_version(WindowsAppSDKVersion::V2)?;
    let runtime = Arc::new(tokio::runtime::Runtime::new().map_err(to_error)?);
    let core = Arc::new(
        AppCore::build(
            Arc::new(Win32RuntimePathProvider),
            Arc::new(RunKeyStartupController::new("MUC-student")),
            Arc::new(NoopEventSink),
        )
        .map_err(to_error)?,
    );

    Application::Start(&ApplicationInitializationCallback::new({
        let runtime = runtime.clone();
        let core = core.clone();
        move |_params: windows_core::Ref<ApplicationInitializationCallbackParams>| {
            let application = Application::new()?;
            let app = WinuiApp::new(core.clone(), runtime.clone())?;
            app.run()?;
            Box::leak(Box::new((application, app)));
            Ok(())
        }
    }))
}

fn force_chinese_locale() {
    for key in ["LANGUAGE", "LANG", "LC_ALL", "LC_CTYPE"] {
        std::env::set_var(key, "zh_CN.UTF-8");
    }
}

fn to_error(err: impl std::fmt::Display) -> windows_core::Error {
    windows_core::Error::new(windows_core::HRESULT(0x80004005u32 as i32), err.to_string())
}
