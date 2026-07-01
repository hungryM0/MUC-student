import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  Activity,
  CheckCircle2,
  CircleGauge,
  Copy,
  Download,
  LogIn,
  Pencil,
  Power,
  Plus,
  RefreshCw,
  Trash2,
  X,
  Users,
  Settings as SettingsIcon,
  Upload,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { WindowFrame } from "@/components/window-frame";
import { MainTitleBar } from "@/components/main-title-bar";
import { SettingsDialog } from "@/components/settings-dialog";
import { AboutDialog } from "@/components/about-dialog";
import { MobileSettings } from "@/components/mobile-settings";
import {
  type AccountDto,
  type AppSnapshotDto,
  addAccount,
  bootstrapApp,
  deleteAccount,
  exportAccountPool,
  importAccountPool,
  loginSelectedAccount,
  logoutLocalDevice,
  readErrorMessage,
  refreshDashboard,
  selectAccount,
  updateAccount,
  FREE_PRODUCT_QUOTA_GB,
} from "@/lib/muc";
import { cn, isAndroid } from "@/lib/utils";
import appIconUrl from "../../src-tauri/icons/icon.svg?url";

function trafficProgressClasses(percent: number, isOnline = true) {
  const c =
    percent >= 100
      ? { bar: "bg-red-700", barOff: "bg-red-700/60", text: "text-red-700" }
      : percent >= 90
        ? { bar: "bg-red-500", barOff: "bg-red-500/60", text: "text-red-500" }
        : percent >= 80
          ? {
              bar: "bg-orange-500",
              barOff: "bg-orange-500/60",
              text: "text-orange-500",
            }
          : percent >= 70
            ? {
                bar: "bg-yellow-500",
                barOff: "bg-yellow-500/60",
                text: "text-yellow-500",
              }
            : {
                bar: "bg-emerald-500",
                barOff: "bg-emerald-500/60",
                text: "text-emerald-500",
              };
  return { bar: isOnline ? c.bar : c.barOff, text: c.text };
}

function parseTrafficValue(text?: string | null) {
  if (!text) return 0;
  const match = text.trim().match(/([0-9]+(?:\.[0-9]+)?)/);
  return match ? Number.parseFloat(match[1]) : 0;
}

function formatTrafficAmount(value: number, digits = 3) {
  return `${value.toFixed(digits)}G`;
}

type RunningAction = "login" | "refresh" | "logout";
type AccountFormState = {
  accountId: string;
  remarkName: string;
  username: string;
  password: string;
};
type AccountPoolDialogMode = "export" | "import";

export default function HomePage() {
  const [snapshot, setSnapshot] = useState<AppSnapshotDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [errorText, setErrorText] = useState("");
  const [displayErrorText, setDisplayErrorText] = useState("");
  const [showError, setShowError] = useState(false);

  useEffect(() => {
    if (errorText) {
      setDisplayErrorText(errorText);
      setShowError(true);
    } else {
      setShowError(false);
      const timer = setTimeout(() => {
        setDisplayErrorText("");
      }, 300);
      return () => clearTimeout(timer);
    }
  }, [errorText]);

  const [selectingId, setSelectingId] = useState("");
  const [loginAccountId, setLoginAccountId] = useState("");
  const [runningAction, setRunningAction] = useState<RunningAction | null>(
    null,
  );
  const [activeTab, setActiveTab] = useState<
    "accounts" | "overview" | "settings"
  >("accounts");
  const [savingAccount, setSavingAccount] = useState(false);
  const [deletingAccountId, setDeletingAccountId] = useState("");
  const [accountToDelete, setAccountToDelete] = useState<AccountDto | null>(
    null,
  );
  const [accountForm, setAccountForm] = useState<AccountFormState | null>(null);
  const [accountPoolMode, setAccountPoolMode] =
    useState<AccountPoolDialogMode | null>(null);
  const [accountPoolCode, setAccountPoolCode] = useState("");
  const [accountPoolPassphrase, setAccountPoolPassphrase] = useState("");
  const [accountPoolBusy, setAccountPoolBusy] = useState(false);
  const [accountPoolResult, setAccountPoolResult] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);

  useEffect(() => {
    const initTrayMenu = async () => {
      try {
        await invoke("update_tray_menu", {
          showText: "显示窗口",
          quitText: "退出",
        });
      } catch {
        setErrorText("托盘菜单初始化失败");
      }
    };

    void initTrayMenu();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;

    void listen<AppSnapshotDto>("muc://state-updated", (event) => {
      if (!disposed) {
        setSnapshot(event.payload);
      }
    }).then((handler) => {
      if (disposed) {
        handler();
      } else {
        unlisten = handler;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    void loadSnapshot();
  }, []);

  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();
    const unlisten = appWindow.onCloseRequested(async (event) => {
      if (snapshot?.preferences.minimizeToTrayOnClose) {
        event.preventDefault();
        await appWindow.hide();
      }
    });

    return () => {
      void unlisten.then((handler) => handler());
    };
  }, [snapshot?.preferences.minimizeToTrayOnClose]);

  const progressPercent = Math.round(snapshot?.poolQuota.progressPercent ?? 0);
  const safeProgress = Math.min(100, Math.max(0, progressPercent));
  const isBusy =
    loading ||
    !!selectingId ||
    !!runningAction ||
    savingAccount ||
    !!deletingAccountId ||
    accountPoolBusy ||
    !!snapshot?.loginState.running ||
    !!snapshot?.refreshState.running;
  const canLogoutLocalDevice = !!snapshot?.accounts.some(
    (account) => account.canLogoutLocalDevice,
  );

  async function loadSnapshot() {
    setLoading(true);
    setErrorText("");
    try {
      setSnapshot(await bootstrapApp());
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setLoading(false);
    }
  }

  async function runSnapshotAction(
    action: RunningAction,
    task: () => Promise<AppSnapshotDto>,
  ) {
    setRunningAction(action);
    setErrorText("");
    try {
      setSnapshot(await task());
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setRunningAction(null);
    }
  }

  async function handleLoginAccount(account: AccountDto) {
    if (isBusy) {
      return;
    }
    setLoginAccountId(account.id);
    setRunningAction("login");
    setErrorText("");
    try {
      if (account.id !== snapshot?.selectedAccountId) {
        setSelectingId(account.id);
        await selectAccount(account.id);
        setSelectingId("");
      }
      setSnapshot(await loginSelectedAccount());
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setSelectingId("");
      setLoginAccountId("");
      setRunningAction(null);
    }
  }

  function openAddAccountForm() {
    setErrorText("");
    setAccountForm({
      accountId: "",
      remarkName: "",
      username: "",
      password: "",
    });
  }

  function openAccountPoolDialog(mode: AccountPoolDialogMode) {
    setErrorText("");
    setAccountPoolMode(mode);
    setAccountPoolCode("");
    setAccountPoolPassphrase("");
    setAccountPoolResult("");
  }

  function closeAccountPoolDialog() {
    if (accountPoolBusy) {
      return;
    }
    setAccountPoolMode(null);
  }

  async function handleExportAccountPool() {
    if (accountPoolBusy) {
      return;
    }
    setAccountPoolBusy(true);
    setErrorText("");
    setAccountPoolResult("");
    try {
      setAccountPoolCode(await exportAccountPool(accountPoolPassphrase));
      setAccountPoolResult("已生成号池码");
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setAccountPoolBusy(false);
    }
  }

  async function handleImportAccountPool() {
    if (accountPoolBusy) {
      return;
    }
    setAccountPoolBusy(true);
    setErrorText("");
    setAccountPoolResult("");
    try {
      const result = await importAccountPool(
        accountPoolCode,
        accountPoolPassphrase,
      );
      setSnapshot(result.snapshot);
      setAccountPoolResult(
        `导入 ${result.importedCount} 个，覆盖 ${result.overwrittenCount} 个`,
      );
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setAccountPoolBusy(false);
    }
  }

  async function copyAccountPoolCode() {
    if (!accountPoolCode.trim()) {
      return;
    }
    try {
      await navigator.clipboard.writeText(accountPoolCode);
      setAccountPoolResult("已复制");
    } catch {
      setErrorText("复制号池码失败");
    }
  }

  function openEditAccountForm(account: AccountDto) {
    setErrorText("");
    setAccountForm({
      accountId: account.id,
      remarkName: account.remarkName,
      username: account.username,
      password: "",
    });
  }

  async function handleSaveAccount() {
    if (!accountForm || savingAccount) {
      return;
    }

    setSavingAccount(true);
    setErrorText("");
    try {
      const nextSnapshot = accountForm.accountId
        ? await updateAccount(
            accountForm.accountId,
            accountForm.remarkName,
            accountForm.username,
            accountForm.password,
          )
        : await addAccount(
            accountForm.remarkName,
            accountForm.username,
            accountForm.password,
          );
      setSnapshot(nextSnapshot);
      setAccountForm(null);
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setSavingAccount(false);
    }
  }

  function requestDeleteAccount(account: AccountDto) {
    if (isBusy || deletingAccountId) {
      return;
    }
    setAccountToDelete(account);
  }

  async function confirmDeleteAccount() {
    if (!accountToDelete || deletingAccountId) {
      return;
    }

    const account = accountToDelete;
    setAccountToDelete(null);
    setDeletingAccountId(account.id);
    setErrorText("");

    // Wait for the exit animation to complete (300ms)
    await new Promise((resolve) => setTimeout(resolve, 300));

    try {
      setSnapshot(await deleteAccount(account.id));
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setDeletingAccountId("");
    }
  }

  return (
    <WindowFrame
      titleBar={
        !isAndroid() ? (
          <MainTitleBar
            onOpenSettings={() => setSettingsOpen(true)}
            onOpenAbout={() => setAboutOpen(true)}
          />
        ) : null
      }
      contentClassName="flex flex-1 overflow-hidden"
    >
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden relative">
        {/* 浮动 Infobar 弹窗 */}
        <div
          className={cn(
            "absolute left-1/2 -translate-x-1/2 z-50 w-[calc(100%-2rem)] max-w-xl transition-all duration-300 ease-out",
            showError
              ? "top-4 opacity-100 scale-100 pointer-events-auto"
              : "-top-20 opacity-0 scale-95 pointer-events-none",
          )}
        >
          {displayErrorText && (
            <div className="border-red-500/30 dark:border-red-500/40 bg-card/95 backdrop-blur-md text-red-600 dark:text-red-400 shadow-xl rounded-xl border px-4 py-3.5 text-sm flex items-start justify-between gap-3">
              <div className="flex items-start gap-2.5 min-w-0 flex-1">
                <span className="h-2 w-2 shrink-0 rounded-full bg-red-500 animate-pulse mt-1.5" />
                <span className="break-words whitespace-pre-wrap select-text font-medium text-left flex-1">
                  {displayErrorText}
                </span>
              </div>
              <button
                onClick={() => setErrorText("")}
                className="text-red-600/70 dark:text-red-400/70 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-500/10 dark:hover:bg-red-500/20 rounded-lg p-1.5 transition-colors shrink-0 mt-0.5"
                aria-label="关闭错误提示"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          )}
        </div>

        <div
          className={cn(
            "shrink-0 px-4 pt-4 md:px-6 md:pt-6",
            isAndroid() && activeTab === "settings" && "hidden",
          )}
        >
          <div className="mx-auto w-full max-w-5xl xl:max-w-7xl 2xl:max-w-[1440px]">
            <header className="flex flex-col sm:flex-row sm:items-center justify-between border-b border-border/40 pb-3 md:pb-4 gap-2 sm:gap-0">
              <div className="flex items-center gap-3 md:gap-4">
                <h1 className="flex items-center gap-2.5 text-xl font-bold tracking-tight text-foreground">
                  <img
                    src={appIconUrl}
                    alt=""
                    className="h-7 w-7 shrink-0 rounded-md"
                  />
                  MUC 校园网拼车
                </h1>
                <div className="bg-border/60 h-4 w-px hidden sm:block" />
              </div>
              <span className="text-muted-foreground text-xs md:hidden">
                IP:{" "}
                {snapshot?.network.ip && snapshot.network.ip !== "unknown"
                  ? snapshot.network.ip
                  : "未知"}
              </span>
              <span className="text-muted-foreground hidden font-mono text-xs sm:inline">
                {snapshot?.network.ip && snapshot.network.ip !== "unknown"
                  ? snapshot.network.ip
                  : "IP 未识别"}
              </span>
            </header>
          </div>
        </div>

        <main
          className={cn(
            "min-w-0 flex-1 overflow-y-auto p-4 md:p-6",
            isAndroid() ? "pb-24" : "",
          )}
        >
          <div className="mx-auto flex w-full max-w-5xl flex-col gap-4 md:gap-6 xl:max-w-7xl 2xl:max-w-[1440px]">
            <div className="flex flex-col gap-4 md:gap-6 md:flex-row">
              {/* 左侧：账号池 */}
              <div
                className={cn(
                  "flex-1 min-w-0",
                  isAndroid() && activeTab !== "accounts"
                    ? "hidden md:block"
                    : "",
                )}
              >
                <Card className="border-border bg-background/95 flex flex-col rounded-xl backdrop-blur-sm overflow-hidden h-full">
                  <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4 border-b border-border/40">
                    <div className="space-y-1">
                      <CardTitle className="flex items-center gap-2 text-lg font-semibold">
                        <Activity className="text-amber-500 h-5 w-5" />
                        账号池
                      </CardTitle>
                      <p className="text-muted-foreground text-xs">
                        管理和切换校园网计费账号
                      </p>
                    </div>
                    <div className="flex gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => openAccountPoolDialog("import")}
                        disabled={isBusy}
                        className="h-8 w-8 px-0 md:w-auto md:px-3 md:gap-1.5"
                      >
                        <Upload className="h-4 w-4" />
                        <span className="hidden md:inline">导入</span>
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => openAccountPoolDialog("export")}
                        disabled={isBusy || !snapshot?.accounts.length}
                        className="h-8 w-8 px-0 md:w-auto md:px-3 md:gap-1.5"
                      >
                        <Download className="h-4 w-4" />
                        <span className="hidden md:inline">导出</span>
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={openAddAccountForm}
                        disabled={isBusy}
                        className="h-8 w-8 px-0 md:w-auto md:px-3 md:gap-1.5"
                      >
                        <Plus className="h-4 w-4" />
                        <span className="hidden md:inline">添加账号</span>
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() =>
                          runSnapshotAction("refresh", refreshDashboard)
                        }
                        disabled={isBusy}
                        className="h-8 w-8 px-0 md:w-auto md:px-3 md:gap-1.5"
                      >
                        <RefreshCw
                          className={cn(
                            "h-3.5 w-3.5",
                            runningAction === "refresh" && "animate-spin",
                          )}
                        />
                        <span className="hidden md:inline">同步数据</span>
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent className="p-4 md:p-6">
                    <div className="max-h-[calc(100vh-240px)] min-h-[300px] overflow-y-auto pr-1">
                      <div className="grid grid-cols-1 gap-3 md:gap-4 xl:grid-cols-2 2xl:grid-cols-3">
                        {snapshot?.accounts.length ? (
                          snapshot.accounts.map((account) => (
                            <AccountRow
                              key={account.id}
                              account={account}
                              selecting={selectingId === account.id}
                              loggingIn={loginAccountId === account.id}
                              disabled={isBusy}
                              deleting={deletingAccountId === account.id}
                              onEdit={() => openEditAccountForm(account)}
                              onDelete={() => requestDeleteAccount(account)}
                              onLogin={() => handleLoginAccount(account)}
                            />
                          ))
                        ) : (
                          <div className="col-span-full text-muted-foreground flex h-48 flex-col items-center justify-center rounded-xl border border-dashed border-border/80 bg-muted/10 text-sm gap-2">
                            {loading ? (
                              <>
                                <RefreshCw className="h-5 w-5 animate-spin text-muted-foreground/60" />
                                <span>正在读取账号列表...</span>
                              </>
                            ) : (
                              <span>暂无账号，请点击上方“添加账号”</span>
                            )}
                          </div>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div>

              {/* 右侧：状态与流量概览 */}
              <div
                className={cn(
                  "w-full md:w-80 shrink-0 flex flex-col gap-6",
                  isAndroid() && activeTab !== "overview"
                    ? "hidden md:flex"
                    : "",
                )}
              >
                <Card className="border-border bg-background/95 rounded-xl backdrop-blur-sm">
                  <CardHeader className="pb-3 border-b border-border/40">
                    <CardTitle className="flex items-center justify-between text-base font-semibold">
                      <div className="flex items-center gap-2">
                        <CircleGauge className="text-emerald-500 h-4.5 w-4.5" />
                        号池概览
                      </div>
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="pt-4 space-y-5">
                    {/* 流量池部分 */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-medium text-muted-foreground">
                          流量池已用
                        </span>
                        <span
                          className={cn(
                            "text-sm font-bold",
                            trafficProgressClasses(safeProgress).text,
                          )}
                        >
                          {safeProgress}%
                        </span>
                      </div>
                      <div className="bg-muted h-2 rounded-full overflow-hidden">
                        <div
                          className={cn(
                            "h-full rounded-full transition-[width] duration-500",
                            trafficProgressClasses(safeProgress).bar,
                          )}
                          style={{ width: `${safeProgress}%` }}
                        />
                      </div>
                      <div className="flex flex-col gap-1 mt-2">
                        <div className="text-xl font-bold tracking-tight">
                          {snapshot?.poolQuota.usedTrafficText ?? "-"}
                        </div>
                        <div className="text-muted-foreground text-xs truncate">
                          {snapshot?.poolQuota.productBalanceText ?? "-"}
                        </div>
                        <div className="text-muted-foreground text-[10px] bg-muted/40 px-2 py-1.5 rounded border border-border/20 mt-1 whitespace-pre-wrap leading-relaxed">
                          {snapshot?.poolQuota.includedPackageText ||
                            "套餐信息为空"}
                        </div>
                      </div>
                    </div>

                    <div className="border-t border-border/40 my-3" />

                    {/* 在线状态部分 */}
                    <div className="space-y-3.5">
                      <div className="flex items-center justify-between gap-3">
                        <span className="text-muted-foreground text-xs">
                          当前在线
                        </span>
                        <span className="max-w-[70%] truncate text-xs font-medium">
                          {snapshot?.currentOnlineAccountId
                            ? snapshot?.accounts.find(
                                (account) =>
                                  account.id ===
                                  snapshot.currentOnlineAccountId,
                              )?.remarkName || "未知账号"
                            : "无在线设备"}
                        </span>
                      </div>

                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground text-xs">
                          最近登录
                        </span>
                        <span className="text-xs font-mono text-muted-foreground">
                          {formatLocalLoginTime(
                            snapshot?.loginState.lastLoginTime,
                          )}
                        </span>
                      </div>

                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground text-xs">
                          已配置账号
                        </span>
                        <span className="text-xs font-medium">
                          {snapshot?.accounts.length || 0}
                        </span>
                      </div>
                    </div>

                    {/* 极其安静的本机下线操作 */}
                    {canLogoutLocalDevice && (
                      <div className="pt-2">
                        <Button
                          variant="ghost"
                          size="sm"
                          disabled={isBusy}
                          onClick={() =>
                            runSnapshotAction("logout", logoutLocalDevice)
                          }
                          className="w-full h-8 gap-1.5 text-xs text-muted-foreground hover:bg-destructive/10 hover:text-destructive hover:border-destructive/30 transition-all border border-border/30"
                        >
                          <Power className="h-3 w-3" />
                          断开校园网
                        </Button>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </div>

              {/* Android 设置 */}
              {isAndroid() && activeTab === "settings" && (
                <div className="w-full flex-1 md:hidden">
                  <MobileSettings />
                </div>
              )}
            </div>
          </div>
        </main>

        {/* Android 底部导航栏 */}
        {isAndroid() && (
          <div className="absolute bottom-0 left-0 right-0 border-t border-border/40 bg-background/95 backdrop-blur-md pb-[env(safe-area-inset-bottom)] z-40 shadow-[0_-4px_16px_rgba(0,0,0,0.05)]">
            <div className="flex h-16 items-center justify-around px-2">
              <button
                onClick={() => setActiveTab("accounts")}
                className={cn(
                  "flex flex-col items-center justify-center w-full h-full gap-1 transition-colors",
                  activeTab === "accounts"
                    ? "text-emerald-500"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <Users className="h-6 w-6" />
                <span className="text-[10px] font-medium">账号池</span>
              </button>
              <button
                onClick={() => setActiveTab("overview")}
                className={cn(
                  "flex flex-col items-center justify-center w-full h-full gap-1 transition-colors",
                  activeTab === "overview"
                    ? "text-emerald-500"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <CircleGauge className="h-6 w-6" />
                <span className="text-[10px] font-medium">概览</span>
              </button>
              <button
                onClick={() => setActiveTab("settings")}
                className={cn(
                  "flex flex-col items-center justify-center w-full h-full gap-1 transition-colors",
                  activeTab === "settings"
                    ? "text-emerald-500"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <SettingsIcon className="h-6 w-6" />
                <span className="text-[10px] font-medium">设置</span>
              </button>
            </div>
          </div>
        )}
        {/* 加载动画遮罩 */}
        <ActionLoadingOverlay
          runningAction={runningAction}
          loginAccountId={loginAccountId}
          accounts={snapshot?.accounts ?? []}
        />
      </div>
      <AccountDialog
        form={accountForm}
        saving={savingAccount}
        onChange={setAccountForm}
        onClose={() => setAccountForm(null)}
        onSave={handleSaveAccount}
      />
      <AccountPoolDialog
        mode={accountPoolMode}
        code={accountPoolCode}
        passphrase={accountPoolPassphrase}
        busy={accountPoolBusy}
        resultText={accountPoolResult}
        onCodeChange={setAccountPoolCode}
        onPassphraseChange={setAccountPoolPassphrase}
        onClose={closeAccountPoolDialog}
        onExport={handleExportAccountPool}
        onImport={handleImportAccountPool}
        onCopy={copyAccountPoolCode}
      />
      <SettingsDialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
      />
      <AboutDialog open={aboutOpen} onClose={() => setAboutOpen(false)} />
      <DeleteConfirmDialog
        account={accountToDelete}
        deleting={!!deletingAccountId}
        onClose={() => setAccountToDelete(null)}
        onConfirm={confirmDeleteAccount}
      />
    </WindowFrame>
  );
}

function formatLocalLoginTime(value?: string | null) {
  if (!value) {
    return "无记录";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "无记录";
  }

  const parts = new Intl.DateTimeFormat("zh-CN", {
    timeZone: "Asia/Shanghai",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).formatToParts(date);
  const getPart = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((part) => part.type === type)?.value ?? "00";

  return `${getPart("year")}-${getPart("month")}-${getPart("day")} ${getPart("hour")}:${getPart("minute")}:${getPart("second")}`;
}

function formatSnapshotSyncText(snapshot: AccountDto["snapshot"]) {
  if (!snapshot) {
    return "未查询";
  }
  if (
    snapshot.statusText === "查询中..." ||
    snapshot.statusText === "查询失败"
  ) {
    return snapshot.statusText;
  }

  const queriedAt = new Date(snapshot.queriedAt);
  if (Number.isNaN(queriedAt.getTime())) {
    return snapshot.statusText || "未查询";
  }

  const elapsedSeconds = Math.max(
    0,
    Math.floor((Date.now() - queriedAt.getTime()) / 1000),
  );
  if (elapsedSeconds < 60) {
    return "刚刚同步";
  }

  const elapsedMinutes = Math.floor(elapsedSeconds / 60);
  if (elapsedMinutes < 60) {
    return `${elapsedMinutes} 分钟前同步`;
  }

  const elapsedHours = Math.floor(elapsedMinutes / 60);
  if (elapsedHours < 24) {
    return `${elapsedHours} 小时前同步`;
  }

  return `${Math.floor(elapsedHours / 24)} 天前同步`;
}

function AccountRow({
  account,
  selecting,
  loggingIn,
  disabled,
  deleting,
  onEdit,
  onDelete,
  onLogin,
}: {
  account: AccountDto;
  selecting: boolean;
  loggingIn: boolean;
  disabled: boolean;
  deleting: boolean;
  onEdit: () => void;
  onDelete: () => void;
  onLogin: () => void;
}) {
  const snapshot = account.snapshot;
  const totalUsed = parseTrafficValue(snapshot?.usedTrafficText);
  const freeQuota = FREE_PRODUCT_QUOTA_GB;
  const packageTotal = parseTrafficValue(snapshot?.packageTotalText);
  const packageAvailable = parseTrafficValue(snapshot?.packageAvailableText);
  const packageUsed = Math.max(0, packageTotal - packageAvailable);
  const freeProgress =
    freeQuota > 0
      ? Math.min(100, Math.max(0, (totalUsed / freeQuota) * 100))
      : 0;
  const packageProgress =
    packageTotal > 0
      ? Math.min(100, Math.max(0, (packageUsed / packageTotal) * 100))
      : 0;
  const accountState = account.isCurrentOnline ? "online" : "idle";

  return (
    <div
      className={cn(
        "grid min-h-20 grid-cols-[1fr_148px] gap-x-4 gap-y-3 rounded-lg border p-4 text-left transition-all duration-300 ease-out animate-slide-in-up",
        accountState === "online"
          ? "border-emerald-500/40 bg-emerald-500/10 shadow-[0_0_12px_rgba(16,185,129,0.15)] ring-1 ring-emerald-500/20 account-card-online"
          : "border-border hover:bg-muted/50 hover:border-muted-foreground/20 hover:shadow-xs",
        deleting && "animate-slide-out-right",
      )}
    >
      <div className="min-w-0 space-y-2">
        <div className="flex min-w-0 items-center gap-2">
          <div className="truncate font-medium">{account.remarkName}</div>
          <AccountStateBadge state={accountState} />
        </div>
        <div className="text-muted-foreground truncate text-sm">
          {account.username}
        </div>
        <div className="text-muted-foreground flex flex-wrap gap-x-4 gap-y-1 text-xs">
          <span>{snapshot?.usedTrafficText ?? "-"}</span>
          <span>{snapshot?.onlineDeviceCountText ?? "0"} 设备</span>
          <span>{formatSnapshotSyncText(snapshot)}</span>
        </div>
      </div>
      <div className="flex flex-col items-end justify-between gap-3">
        <div className="w-full text-right">
          <div
            className={cn(
              "font-semibold",
              trafficProgressClasses(Math.round(freeProgress)).text,
            )}
          >
            {Math.round(freeProgress)}%
          </div>
          <div className="text-muted-foreground mt-1 truncate text-xs">
            {formatTrafficAmount(totalUsed)} / {formatTrafficAmount(freeQuota)}
          </div>
        </div>
        <div className="grid w-full grid-cols-[1fr_32px_32px] gap-1.5">
          <Button
            type="button"
            size="sm"
            disabled={disabled}
            onClick={onLogin}
            className={cn(
              "h-8 w-full transition-all duration-300 relative overflow-hidden",
              (loggingIn || selecting) &&
                "bg-emerald-600 hover:bg-emerald-600 text-white",
            )}
          >
            <span className="flex items-center justify-center gap-1.5 transition-all duration-300">
              {loggingIn || selecting ? (
                <RefreshCw className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <LogIn className="h-3.5 w-3.5" />
              )}
              {loggingIn || selecting ? "登录中" : "登录"}
            </span>
          </Button>
          <Button
            type="button"
            variant="outline"
            size="icon"
            disabled={disabled}
            onClick={onEdit}
            aria-label="编辑账号"
            className="h-8 w-8 transition-transform duration-200 hover:scale-105 active:scale-95"
          >
            <Pencil className="h-3.5 w-3.5" />
          </Button>
          <Button
            type="button"
            variant="outline"
            size="icon"
            disabled={disabled}
            onClick={onDelete}
            aria-label="删除账号"
            className="h-8 w-8 transition-transform duration-200 hover:scale-105 active:scale-95 hover:text-destructive"
          >
            <Trash2
              className={cn("h-3.5 w-3.5", deleting && "animate-pulse")}
            />
          </Button>
        </div>
      </div>
      {/* 进度条 */}
      <div className="col-span-2 mt-1">
        <div className="grid gap-2">
          <div className="grid gap-1">
            <div className="flex justify-between text-[11px] text-muted-foreground">
              <span>免费包</span>
              <span>{Math.round(freeProgress)}%</span>
            </div>
            <div
              className={cn(
                "h-1.5 w-full rounded-full overflow-hidden",
                accountState === "online"
                  ? "bg-emerald-500/8 dark:bg-zinc-950/30"
                  : "bg-muted",
              )}
            >
              <div
                className={cn(
                  "h-full rounded-full transition-[width] duration-500 ease-out",
                  trafficProgressClasses(
                    Math.round(freeProgress),
                    accountState === "online",
                  ).bar,
                )}
                style={{ width: `${freeProgress}%` }}
              />
            </div>
            <div className="flex justify-between text-[11px] text-muted-foreground">
              <span>{snapshot?.usedTrafficText ?? "-"}</span>
              <span>{formatTrafficAmount(freeQuota)}</span>
            </div>
          </div>

          {packageTotal > 0 && (
            <div className="grid gap-1">
              <div className="flex justify-between text-[11px] text-muted-foreground">
                <span>套餐流量</span>
                <span>{Math.round(packageProgress)}%</span>
              </div>
              <div className="bg-muted h-1.5 rounded-full overflow-hidden">
                <div
                  className={cn(
                    "h-full rounded-full transition-[width] duration-500 ease-out bg-sky-500",
                  )}
                  style={{ width: `${packageProgress}%` }}
                />
              </div>
              <div className="flex justify-between text-[11px] text-muted-foreground">
                <span>{formatTrafficAmount(packageUsed)}</span>
                <span>{snapshot?.packageTotalText || "-"}</span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function AccountStateBadge({ state }: { state: "online" | "idle" }) {
  const isOnline = state === "online";
  return (
    <span
      className={cn(
        "inline-flex shrink-0 items-center gap-1 rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs text-emerald-600 transition-all duration-300 transform origin-left",
        isOnline
          ? "opacity-100 scale-100 max-w-[100px] translate-x-0"
          : "opacity-0 scale-90 max-w-0 translate-x-[-10px] pointer-events-none overflow-hidden",
      )}
    >
      <CheckCircle2 className="h-3 w-3 shrink-0" />
      <span className="shrink-0">在线</span>
    </span>
  );
}

function AccountDialog({
  form,
  saving,
  onChange,
  onClose,
  onSave,
}: {
  form: AccountFormState | null;
  saving: boolean;
  onChange: (form: AccountFormState | null) => void;
  onClose: () => void;
  onSave: () => void;
}) {
  const [localForm, setLocalForm] = useState<AccountFormState | null>(null);

  useEffect(() => {
    if (form) {
      setLocalForm(form);
    }
  }, [form]);

  if (!localForm) {
    return null;
  }

  const isEditing = !!localForm.accountId;
  const canSave =
    !!localForm.remarkName.trim() &&
    !!localForm.username.trim() &&
    (isEditing || !!localForm.password.trim());

  return (
    <Dialog open={!!form} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEditing ? "编辑账号" : "添加账号"}</DialogTitle>
        </DialogHeader>

        <div className="grid gap-3 py-1">
          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">备注名</span>
            <Input
              value={localForm.remarkName}
              onChange={(event) => {
                const updated = {
                  ...localForm,
                  remarkName: event.target.value,
                };
                setLocalForm(updated);
                onChange(updated);
              }}
              autoFocus
            />
          </label>

          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">账号</span>
            <Input
              value={localForm.username}
              onChange={(event) => {
                const updated = { ...localForm, username: event.target.value };
                setLocalForm(updated);
                onChange(updated);
              }}
            />
          </label>

          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">密码</span>
            <Input
              type="password"
              value={localForm.password}
              placeholder={isEditing ? "留空则不修改" : ""}
              onChange={(event) => {
                const updated = { ...localForm, password: event.target.value };
                setLocalForm(updated);
                onChange(updated);
              }}
            />
          </label>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={saving}>
            取消
          </Button>
          <Button onClick={onSave} disabled={saving || !canSave}>
            {saving ? "保存中" : "保存"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function AccountPoolDialog({
  mode,
  code,
  passphrase,
  busy,
  resultText,
  onCodeChange,
  onPassphraseChange,
  onClose,
  onExport,
  onImport,
  onCopy,
}: {
  mode: AccountPoolDialogMode | null;
  code: string;
  passphrase: string;
  busy: boolean;
  resultText: string;
  onCodeChange: (value: string) => void;
  onPassphraseChange: (value: string) => void;
  onClose: () => void;
  onExport: () => void;
  onImport: () => void;
  onCopy: () => void;
}) {
  if (!mode) {
    return null;
  }

  const isExport = mode === "export";
  const canSubmit = !!passphrase.trim() && (isExport || !!code.trim()) && !busy;

  return (
    <Dialog open={!!mode} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>{isExport ? "导出号池" : "导入号池"}</DialogTitle>
        </DialogHeader>

        <div className="grid gap-3 py-1">
          {!isExport && (
            <label className="grid gap-1.5 text-sm">
              <span className="font-medium">号池码</span>
              <textarea
                value={code}
                onChange={(event) => onCodeChange(event.target.value)}
                className="min-h-28 w-full resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30"
              />
            </label>
          )}

          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">号池口令</span>
            <Input
              type="password"
              value={passphrase}
              onChange={(event) => onPassphraseChange(event.target.value)}
              autoFocus
            />
          </label>

          {isExport && (
            <label className="grid gap-1.5 text-sm">
              <span className="font-medium">号池码</span>
              <div className="grid grid-cols-[1fr_36px] gap-2">
                <textarea
                  value={code}
                  readOnly
                  className="min-h-28 w-full resize-none rounded-md border border-input bg-muted/30 px-3 py-2 text-sm shadow-xs outline-none"
                />
                <Button
                  type="button"
                  variant="outline"
                  size="icon-sm"
                  disabled={!code.trim()}
                  onClick={onCopy}
                  aria-label="复制号池码"
                  className="h-9 w-9"
                >
                  <Copy className="h-4 w-4" />
                </Button>
              </div>
            </label>
          )}

          {resultText && (
            <div className="rounded-md border border-emerald-500/20 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-700 dark:text-emerald-300">
              {resultText}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={busy}>
            关闭
          </Button>
          <Button
            onClick={isExport ? onExport : onImport}
            disabled={!canSubmit}
          >
            {busy ? "处理中" : isExport ? "生成" : "导入"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

interface DeleteConfirmDialogProps {
  account: AccountDto | null;
  deleting: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

function DeleteConfirmDialog({
  account,
  deleting,
  onClose,
  onConfirm,
}: DeleteConfirmDialogProps) {
  if (!account) {
    return null;
  }

  return (
    <Dialog open={!!account} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-xs">
        <DialogHeader>
          <DialogTitle>删除确认</DialogTitle>
        </DialogHeader>

        <div className="py-2 text-sm text-muted-foreground">
          确定要删除账号“
          <span className="font-semibold text-foreground">
            {account.remarkName}
          </span>
          ”吗？
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            size="sm"
            onClick={onClose}
            disabled={deleting}
          >
            取消
          </Button>
          <Button
            variant="destructive"
            size="sm"
            onClick={onConfirm}
            disabled={deleting}
          >
            {deleting ? "删除中" : "确认删除"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function ActionLoadingOverlay({
  runningAction,
  loginAccountId,
  accounts,
}: {
  runningAction: RunningAction | null;
  loginAccountId: string;
  accounts: AccountDto[];
}) {
  if (!runningAction) return null;

  const targetAccount = accounts.find((a) => a.id === loginAccountId);
  const remarkName = targetAccount?.remarkName || targetAccount?.username;

  // 极简动作配置，仅配置主题色圆环和状态文字
  let config = {
    themeColor: "border-t-emerald-500",
    title: remarkName ? `正在登录 ${remarkName}...` : "正在登录校园网...",
    subtitle: "此过程可能需要一些时间，请稍候",
  };

  if (runningAction === "refresh") {
    config = {
      themeColor: "border-t-amber-500",
      title: "正在同步数据...",
      subtitle: "正在拉取最新的套餐流量与在线设备信息",
    };
  } else if (runningAction === "logout") {
    config = {
      themeColor: "border-t-red-500",
      title: "正在断开校园网...",
      subtitle: "正在向 Portal 认证服务器请求断开连接",
    };
  }

  return (
    <div className="absolute inset-0 z-50 flex items-center justify-center bg-background/35 backdrop-blur-[4px] transition-all duration-300 animate-fade-in">
      <div className="bg-card/90 border border-border/40 shadow-xl rounded-xl p-6 flex flex-col items-center gap-4 max-w-[260px] w-[90%] backdrop-blur-md transition-all duration-300 animate-scale-in-simple">
        {/* 极简优雅的 App Logo 旋转环 */}
        <div className="relative flex items-center justify-center h-14 w-14">
          {/* 极其纤细的外旋转环 */}
          <div
            className={cn(
              "absolute -inset-1.5 rounded-full border border-muted/40 animate-spin",
              config.themeColor,
            )}
          />
          {/* 应用 Logo，伴随温和呼吸效果 */}
          <img
            src={appIconUrl}
            alt="App Logo"
            className="h-10 w-10 shrink-0 rounded-lg animate-pulse"
          />
        </div>

        {/* 简约文案 */}
        <div className="text-center space-y-1">
          <h3 className="text-xs font-semibold text-foreground/90 tracking-wide">
            {config.title}
          </h3>
          <p className="text-[10px] text-muted-foreground leading-normal max-w-[180px] mx-auto">
            {config.subtitle}
          </p>
        </div>
      </div>
    </div>
  );
}
