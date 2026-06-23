use std::sync::{Arc, RwLock};

use muc_student_core::application::{AppCore, AppSnapshotDto};
use windows_core::{Interface, Result, HSTRING};
use winui3::Microsoft::UI::Dispatching::{DispatcherQueue, DispatcherQueueHandler};
use winui3::Microsoft::UI::Xaml::Controls::{
    Button, Canvas, ComboBox, ComboBoxItem, ScrollViewer, SelectionChangedEventArgs, TextBlock,
};
use winui3::Microsoft::UI::Xaml::{RoutedEventArgs, RoutedEventHandler, UIElement, Window};

pub struct WinuiApp {
    core: Arc<AppCore>,
    runtime: Arc<tokio::runtime::Runtime>,
    window: Window,
    dispatcher: DispatcherQueue,
    state: Arc<UiState>,
}

struct UiState {
    quota_title: TextBlock,
    quota_text: TextBlock,
    progress_text: TextBlock,
    account_combo: ComboBox,
    accounts_panel: Canvas,
    status_text: TextBlock,
    login_button: Button,
    refresh_button: Button,
    account_ids: RwLock<Vec<String>>,
    rendering_accounts: RwLock<bool>,
}

impl WinuiApp {
    pub fn new(core: Arc<AppCore>, runtime: Arc<tokio::runtime::Runtime>) -> Result<Self> {
        let window = Window::new()?;
        window.SetTitle(&HSTRING::from("MUC-student"))?;
        let dispatcher = window.DispatcherQueue()?;

        let state = Arc::new(build_content(&window)?);

        Ok(Self {
            core,
            runtime,
            window,
            dispatcher,
            state,
        })
    }

    pub fn run(&self) -> Result<()> {
        self.bind_events()?;
        self.window.Activate()?;
        Ok(())
    }

    fn bind_events(&self) -> Result<()> {
        self.state.login_button.Click(&RoutedEventHandler::new({
            let core = self.core.clone();
            let runtime = self.runtime.clone();
            let dispatcher = self.dispatcher.clone();
            let state = self.state.clone();
            move |_sender, _args: windows_core::Ref<RoutedEventArgs>| {
                state.status_text.SetText(&HSTRING::from("正在登录..."))?;
                let core = core.clone();
                spawn_snapshot(
                    runtime.clone(),
                    dispatcher.clone(),
                    state.clone(),
                    async move { core.login_selected_account().await },
                );
                Ok(())
            }
        }))?;

        self.state.refresh_button.Click(&RoutedEventHandler::new({
            let core = self.core.clone();
            let runtime = self.runtime.clone();
            let dispatcher = self.dispatcher.clone();
            let state = self.state.clone();
            move |_sender, _args: windows_core::Ref<RoutedEventArgs>| {
                state.status_text.SetText(&HSTRING::from("正在刷新..."))?;
                let core = core.clone();
                spawn_snapshot(
                    runtime.clone(),
                    dispatcher.clone(),
                    state.clone(),
                    async move { core.refresh_dashboard().await },
                );
                Ok(())
            }
        }))?;

        self.state.account_combo.SelectionChanged(
            &winui3::Microsoft::UI::Xaml::Controls::SelectionChangedEventHandler::new({
                let core = self.core.clone();
                let runtime = self.runtime.clone();
                let dispatcher = self.dispatcher.clone();
                let state = self.state.clone();
                move |_sender: windows_core::Ref<windows_core::IInspectable>,
                      _args: windows_core::Ref<SelectionChangedEventArgs>| {
                    if *state
                        .rendering_accounts
                        .read()
                        .expect("ui state lock poisoned")
                    {
                        return Ok(());
                    }
                    let index = state.account_combo.SelectedIndex()?;
                    if index < 0 {
                        return Ok(());
                    }
                    let account_id = state
                        .account_ids
                        .read()
                        .expect("ui state lock poisoned")
                        .get(index as usize)
                        .cloned();
                    let Some(account_id) = account_id else {
                        return Ok(());
                    };
                    state
                        .status_text
                        .SetText(&HSTRING::from("正在切换账号..."))?;
                    let core = core.clone();
                    spawn_snapshot(
                        runtime.clone(),
                        dispatcher.clone(),
                        state.clone(),
                        async move { core.select_account(account_id).await },
                    );
                    Ok(())
                }
            }),
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    fn bootstrap(&self) {
        let core = self.core.clone();
        spawn_snapshot(
            self.runtime.clone(),
            self.dispatcher.clone(),
            self.state.clone(),
            async move { core.bootstrap_app().await },
        );
    }
}

fn build_content(window: &Window) -> Result<UiState> {
    let root = Canvas::new()?;
    root.SetWidth(920.0)?;
    root.SetHeight(640.0)?;

    let app_title = title("MUC 校园网", 18.0)?;
    canvas_append(&root, &app_title, 24.0, 24.0)?;
    canvas_append(&root, &button("首页")?, 24.0, 72.0)?;
    canvas_append(&root, &button("账号管理")?, 24.0, 120.0)?;
    canvas_append(&root, &button("设置")?, 24.0, 168.0)?;

    let quota_title = title("当前流量池情况", 16.0)?;
    let quota_text = text("0 GB / 0 GB")?;
    let progress_text = text("进度：0%")?;

    let account_combo = ComboBox::new()?;
    account_combo.SetWidth(220.0)?;

    let status_text = text("就绪")?;
    let accounts_panel = Canvas::new()?;
    accounts_panel.SetWidth(560.0)?;
    accounts_panel.SetHeight(280.0)?;

    let scroll = ScrollViewer::new()?;
    scroll.SetContent(&accounts_panel)?;

    let login_button = button("登录")?;
    let refresh_button = button("刷新状态")?;

    canvas_append(&root, &quota_title, 280.0, 32.0)?;
    canvas_append(&root, &quota_text, 280.0, 72.0)?;
    canvas_append(&root, &progress_text, 280.0, 112.0)?;
    canvas_append(&root, &account_combo, 280.0, 144.0)?;
    canvas_append(&root, &login_button, 520.0, 144.0)?;
    canvas_append(&root, &refresh_button, 600.0, 144.0)?;
    canvas_append(&root, &status_text, 280.0, 192.0)?;
    canvas_append(&root, &scroll, 280.0, 232.0)?;

    window.SetContent(&root)?;

    Ok(UiState {
        quota_title,
        quota_text,
        progress_text,
        account_combo,
        accounts_panel,
        status_text,
        login_button,
        refresh_button,
        account_ids: RwLock::new(Vec::new()),
        rendering_accounts: RwLock::new(false),
    })
}

fn spawn_snapshot<F>(
    runtime: Arc<tokio::runtime::Runtime>,
    dispatcher: DispatcherQueue,
    state: Arc<UiState>,
    future: F,
) where
    F: std::future::Future<Output = muc_student_core::application::AppResult<AppSnapshotDto>>
        + Send
        + 'static,
{
    runtime.spawn(async move {
        let result = future.await;
        let _ = dispatcher.TryEnqueue(&DispatcherQueueHandler::new(move || {
            match &result {
                Ok(snapshot) => render_snapshot(&state, snapshot)?,
                Err(err) => state
                    .status_text
                    .SetText(&HSTRING::from(format!("失败：{err}")))?,
            }
            Ok(())
        }));
    });
}

fn render_snapshot(state: &UiState, snapshot: &AppSnapshotDto) -> Result<()> {
    let percent = snapshot.pool_quota.progress_percent.unwrap_or(0.0) * 100.0;
    state
        .quota_title
        .SetText(&HSTRING::from("当前流量池情况"))?;
    state.quota_text.SetText(&HSTRING::from(format!(
        "{} / {}",
        snapshot.pool_quota.used_traffic_text, snapshot.pool_quota.product_balance_text
    )))?;
    state.progress_text.SetText(&HSTRING::from(format!(
        "进度：{:.0}%",
        percent.clamp(0.0, 100.0)
    )))?;
    state.status_text.SetText(&HSTRING::from(
        snapshot.login_state.message.clone().if_empty("就绪"),
    ))?;

    {
        let mut rendering = state
            .rendering_accounts
            .write()
            .expect("ui state lock poisoned");
        *rendering = true;
        let items = state.account_combo.Items()?;
        items.Clear()?;
        {
            let mut account_ids = state.account_ids.write().expect("ui state lock poisoned");
            account_ids.clear();
            for account in &snapshot.accounts {
                let label = format!("{}（{}）", account.remark_name, account.username);
                items.Append(&combo_item(&label)?)?;
                account_ids.push(account.id.clone());
            }
        }
        let selected_index = snapshot
            .accounts
            .iter()
            .position(|account| account.id == snapshot.selected_account_id)
            .map(|index| index as i32)
            .unwrap_or(-1);
        state.account_combo.SetSelectedIndex(selected_index)?;
        *rendering = false;
    }

    let children = state.accounts_panel.Children()?;
    children.Clear()?;
    for (index, account) in snapshot.accounts.iter().enumerate() {
        canvas_append(
            &state.accounts_panel,
            &account_card(snapshot, account)?,
            0.0,
            index as f64 * 96.0,
        )?;
    }
    Ok(())
}

fn account_card(
    snapshot: &AppSnapshotDto,
    account: &muc_student_core::application::AccountDto,
) -> Result<Canvas> {
    let card = Canvas::new()?;
    card.SetWidth(520.0)?;
    card.SetHeight(88.0)?;
    canvas_append(&card, &title(&account.remark_name, 15.0)?, 0.0, 0.0)?;
    canvas_append(
        &card,
        &text(&format!("账号：{}", account.username))?,
        0.0,
        24.0,
    )?;
    let used = account
        .snapshot
        .as_ref()
        .map(|item| item.used_traffic_text.clone())
        .unwrap_or_else(|| "未查询".to_string());
    let devices = account
        .snapshot
        .as_ref()
        .map(|item| item.online_device_count_text.clone())
        .unwrap_or_else(|| "0".to_string());
    canvas_append(
        &card,
        &text(&format!("已用：{used} | 设备：{devices}"))?,
        0.0,
        48.0,
    )?;
    if snapshot.current_online_account_id == account.id {
        canvas_append(&card, &text("当前在线")?, 420.0, 0.0)?;
    }
    Ok(card)
}

fn canvas_append(parent: &Canvas, child: &impl Interface, left: f64, top: f64) -> Result<()> {
    let element: UIElement = child.cast()?;
    Canvas::SetLeft(&element, left)?;
    Canvas::SetTop(&element, top)?;
    parent.Children()?.Append(&element)
}

fn text(value: &str) -> Result<TextBlock> {
    let block = TextBlock::new()?;
    block.SetText(&HSTRING::from(value))?;
    Ok(block)
}

fn title(value: &str, size: f64) -> Result<TextBlock> {
    let block = text(value)?;
    block.SetFontSize(size)?;
    Ok(block)
}

fn button(value: &str) -> Result<Button> {
    let button = Button::new()?;
    let content = text(value)?;
    button.SetContent(&content)?;
    Ok(button)
}

fn combo_item(value: &str) -> Result<ComboBoxItem> {
    let item = ComboBoxItem::new()?;
    let content = text(value)?;
    item.SetContent(&content)?;
    Ok(item)
}

trait IfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}

impl IfEmpty for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}
