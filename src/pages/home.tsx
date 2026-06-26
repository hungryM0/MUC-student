import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  Activity,
  CheckCircle2,
  CircleGauge,
  LogIn,
  Pencil,
  Power,
  Plus,
  RefreshCw,
  Trash2,
  Wifi,
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
import {
  type AccountDto,
  type AppSnapshotDto,
  addAccount,
  bootstrapApp,
  deleteAccount,
  loginSelectedAccount,
  logoutLocalDevice,
  readErrorMessage,
  refreshDashboard,
  selectAccount,
  updateAccount,
} from "@/lib/muc";
import { cn } from "@/lib/utils";

type RunningAction = "login" | "refresh" | "logout";
type AccountFormState = {
  accountId: string;
  remarkName: string;
  username: string;
  password: string;
};

export default function HomePage() {
  const [snapshot, setSnapshot] = useState<AppSnapshotDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [errorText, setErrorText] = useState("");
  const [selectingId, setSelectingId] = useState("");
  const [loginAccountId, setLoginAccountId] = useState("");
  const [runningAction, setRunningAction] = useState<RunningAction | null>(
    null,
  );
  const [savingAccount, setSavingAccount] = useState(false);
  const [deletingAccountId, setDeletingAccountId] = useState("");
  const [accountForm, setAccountForm] = useState<AccountFormState | null>(null);

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

  async function handleDeleteAccount(account: AccountDto) {
    if (isBusy || deletingAccountId) {
      return;
    }

    const confirmed = window.confirm(`删除账号“${account.remarkName}”？`);
    if (!confirmed) {
      return;
    }

    setDeletingAccountId(account.id);
    setErrorText("");
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
      titleBar={<MainTitleBar />}
      contentClassName="flex flex-1 overflow-hidden bg-[radial-gradient(circle_at_top_left,rgba(14,165,233,0.12),transparent_35%),linear-gradient(135deg,rgba(16,185,129,0.06),transparent_45%)]"
    >
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <main className="min-w-0 flex-1 overflow-y-auto p-6">
          <div className="mx-auto flex max-w-4xl flex-col gap-6">
            {/* 顶栏 Header */}
            <header className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between border-b border-border/40 pb-5">
              <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
                <div className="space-y-1">
                  <h1 className="text-2xl font-bold tracking-tight bg-gradient-to-r from-emerald-500 to-sky-500 bg-clip-text text-transparent">
                    MUC 校园网拼车
                  </h1>
                  <p className="text-muted-foreground text-xs">
                    多账号管理与智能状态同步
                  </p>
                </div>
                <div className="bg-border/60 hidden h-8 w-px sm:block" />
                <div className="flex items-center gap-3">
                  <StatusPill
                    tone={snapshot?.network.isOnline ? "green" : "amber"}
                    label={snapshot?.network.statusText || "IP 未识别"}
                  />
                  <span className="text-muted-foreground font-mono text-xs">
                    {snapshot?.network.ip && snapshot.network.ip !== "unknown"
                      ? snapshot.network.ip
                      : "IP 未识别"}
                  </span>
                </div>
              </div>

              {/* 全局动作 */}
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={isBusy || !snapshot?.accounts.length}
                  onClick={() => runSnapshotAction("refresh", refreshDashboard)}
                  className="h-9 gap-2 shadow-sm"
                >
                  <RefreshCw
                    className={cn(
                      "h-4 w-4",
                      runningAction === "refresh" && "animate-spin",
                    )}
                  />
                  刷新流量
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={isBusy || !canLogoutLocalDevice}
                  onClick={() => runSnapshotAction("logout", logoutLocalDevice)}
                  className="h-9 gap-2 shadow-sm text-destructive hover:bg-destructive/10 hover:text-destructive"
                >
                  <Power
                    className={cn(
                      "h-4 w-4",
                      runningAction === "logout" && "animate-pulse",
                    )}
                  />
                  本机下线
                </Button>
              </div>
            </header>

            {/* 核心指标卡片区 */}
            <section className="grid grid-cols-1 gap-5 sm:grid-cols-2">
              {/* 流量池卡片 */}
              <Card className="border-border/70 bg-background/90 rounded-xl shadow-sm backdrop-blur-sm">
                <CardHeader className="pb-3">
                  <CardTitle className="flex items-center gap-2 text-base font-medium">
                    <CircleGauge className="text-emerald-500 h-4 w-4" />
                    流量池
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-end justify-between gap-4">
                    <div className="min-w-0">
                      <div className="truncate text-3xl font-bold tracking-tight">
                        {snapshot?.poolQuota.usedTrafficText ?? "-"}
                      </div>
                      <div className="text-muted-foreground mt-1 truncate text-xs">
                        {snapshot?.poolQuota.productBalanceText ?? "-"}
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-2xl font-bold text-emerald-500">
                        {safeProgress}%
                      </div>
                      <div className="text-muted-foreground text-xs">已用</div>
                    </div>
                  </div>
                  <div className="bg-muted h-2 rounded-full overflow-hidden">
                    <div
                      className="h-full rounded-full bg-gradient-to-r from-emerald-500 to-teal-500 transition-[width] duration-500"
                      style={{ width: `${safeProgress}%` }}
                    />
                  </div>
                  <div className="text-muted-foreground truncate text-xs bg-muted/30 px-2.5 py-1.5 rounded-md border border-border/30">
                    {snapshot?.poolQuota.includedPackageText || "套餐信息为空"}
                  </div>
                </CardContent>
              </Card>

              {/* 当前状态卡片 */}
              <Card className="border-border/70 bg-background/90 rounded-xl shadow-sm backdrop-blur-sm">
                <CardHeader className="pb-3">
                  <CardTitle className="flex items-center gap-2 text-base font-medium">
                    <Wifi className="text-sky-500 h-4 w-4" />
                    当前状态
                  </CardTitle>
                </CardHeader>
                <CardContent className="flex flex-col justify-between h-[134px]">
                  <div className="space-y-3.5">
                    <Metric
                      label="在线账号"
                      value={
                        snapshot?.accounts.find(
                          (account) =>
                            account.id === snapshot.currentOnlineAccountId,
                        )?.remarkName || "无在线设备"
                      }
                      isOnline={!!snapshot?.currentOnlineAccountId}
                    />
                    <Metric
                      label="最近登录"
                      value={
                        snapshot?.loginState.resultText ||
                        snapshot?.loginState.message ||
                        "无记录"
                      }
                    />
                  </div>
                  <div className="text-muted-foreground text-xs border-t border-border/40 pt-3">
                    配置了 {snapshot?.accounts.length || 0} 个拼车账号
                  </div>
                </CardContent>
              </Card>
            </section>

            {errorText && (
              <div className="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border px-4 py-3 text-sm flex items-center gap-2">
                <span className="h-1.5 w-1.5 rounded-full bg-destructive" />
                {errorText}
              </div>
            )}

            {/* 账号池卡片 */}
            <Card className="border-border/70 bg-background/90 flex min-h-0 flex-col rounded-xl shadow-sm backdrop-blur-sm overflow-hidden">
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
                    onClick={openAddAccountForm}
                    disabled={isBusy}
                    className="h-8 gap-1.5"
                  >
                    <Plus className="h-4 w-4" />
                    添加账号
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={loadSnapshot}
                    disabled={isBusy}
                    className="h-8 gap-1.5"
                  >
                    <RefreshCw
                      className={cn("h-3.5 w-3.5", loading && "animate-spin")}
                    />
                    同步数据
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="p-6">
                <div className="max-h-[calc(100vh-390px)] min-h-[220px] overflow-y-auto pr-1">
                  <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                    {snapshot?.accounts.length ? (
                      snapshot.accounts.map((account) => (
                        <AccountRow
                          key={account.id}
                          account={account}
                          selected={account.id === snapshot.selectedAccountId}
                          selecting={selectingId === account.id}
                          loggingIn={loginAccountId === account.id}
                          disabled={isBusy}
                          deleting={deletingAccountId === account.id}
                          onEdit={() => openEditAccountForm(account)}
                          onDelete={() => handleDeleteAccount(account)}
                          onLogin={() => handleLoginAccount(account)}
                        />
                      ))
                    ) : (
                      <div className="col-span-2 text-muted-foreground flex h-40 flex-col items-center justify-center rounded-xl border border-dashed border-border/80 bg-muted/10 text-sm gap-2">
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
        </main>
      </div>
      <AccountDialog
        form={accountForm}
        saving={savingAccount}
        onChange={setAccountForm}
        onClose={() => setAccountForm(null)}
        onSave={handleSaveAccount}
      />
    </WindowFrame>
  );
}

function StatusPill({
  label,
  tone,
}: {
  label: string;
  tone: "green" | "amber";
}) {
  return (
    <div
      className={cn(
        "inline-flex max-w-full items-center gap-2 rounded-full border px-3 py-1.5 text-xs",
        tone === "green"
          ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-600"
          : "border-amber-500/30 bg-amber-500/10 text-amber-600",
      )}
    >
      <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-current" />
      <span className="truncate">{label}</span>
    </div>
  );
}

function Metric({
  label,
  value,
  isOnline = false,
}: {
  label: string;
  value: string;
  isOnline?: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-muted-foreground text-sm">{label}</span>
      <div className="flex items-center gap-1.5 truncate">
        {isOnline && (
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
        )}
        <span className="truncate text-sm font-medium">{value}</span>
      </div>
    </div>
  );
}

function AccountRow({
  account,
  selected,
  selecting,
  loggingIn,
  disabled,
  deleting,
  onEdit,
  onDelete,
  onLogin,
}: {
  account: AccountDto;
  selected: boolean;
  selecting: boolean;
  loggingIn: boolean;
  disabled: boolean;
  deleting: boolean;
  onEdit: () => void;
  onDelete: () => void;
  onLogin: () => void;
}) {
  const snapshot = account.snapshot;
  const progress = Math.round(snapshot?.progressPercent ?? 0);
  const accountState = account.isCurrentOnline
    ? "online"
    : selected
      ? "selected"
      : "idle";

  return (
    <div
      className={cn(
        "grid min-h-20 grid-cols-[1fr_148px] gap-4 rounded-lg border p-4 text-left transition-colors",
        accountState === "online"
          ? "border-emerald-500/40 bg-emerald-500/10"
          : "border-border hover:bg-muted/50",
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
          <span>{snapshot?.statusText ?? "未查询"}</span>
        </div>
      </div>
      <div className="flex flex-col items-end justify-between gap-3">
        <div className="w-full text-right">
          <div className="font-semibold">
            {Math.min(100, Math.max(0, progress))}%
          </div>
          <div className="text-muted-foreground mt-1 truncate text-xs">
            {snapshot?.productBalanceText ?? "-"}
          </div>
        </div>
        <div className="grid w-full grid-cols-[1fr_32px_32px] gap-1.5">
          <Button
            type="button"
            size="sm"
            disabled={disabled}
            onClick={onLogin}
            className="h-8 w-full"
          >
            <LogIn
              className={cn(
                "h-3.5 w-3.5",
                (loggingIn || selecting) && "animate-pulse",
              )}
            />
            {loggingIn || selecting ? "登录中" : "登录"}
          </Button>
          <Button
            type="button"
            variant="outline"
            size="icon"
            disabled={disabled}
            onClick={onEdit}
            aria-label="编辑账号"
            className="h-8 w-8"
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
            className="h-8 w-8"
          >
            <Trash2
              className={cn("h-3.5 w-3.5", deleting && "animate-pulse")}
            />
          </Button>
        </div>
      </div>
    </div>
  );
}

function AccountStateBadge({
  state,
}: {
  state: "online" | "selected" | "idle";
}) {
  if (state === "idle") {
    return null;
  }

  if (state === "online") {
    return (
      <span className="inline-flex shrink-0 items-center gap-1 rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs text-emerald-600">
        <CheckCircle2 className="h-3 w-3" />
        在线
      </span>
    );
  }

  return (
    <span className="text-muted-foreground inline-flex shrink-0 items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-xs">
      <Wifi className="h-3 w-3" />
      已选
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
  if (!form) {
    return null;
  }

  const isEditing = !!form.accountId;
  const canSave =
    form.remarkName.trim() &&
    form.username.trim() &&
    (isEditing || form.password.trim());

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEditing ? "编辑账号" : "添加账号"}</DialogTitle>
        </DialogHeader>

        <div className="grid gap-3 py-1">
          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">备注名</span>
            <Input
              value={form.remarkName}
              onChange={(event) =>
                onChange({ ...form, remarkName: event.target.value })
              }
              autoFocus
            />
          </label>

          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">账号</span>
            <Input
              value={form.username}
              onChange={(event) =>
                onChange({ ...form, username: event.target.value })
              }
            />
          </label>

          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">密码</span>
            <Input
              type="password"
              value={form.password}
              placeholder={isEditing ? "留空则不修改" : ""}
              onChange={(event) =>
                onChange({ ...form, password: event.target.value })
              }
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
