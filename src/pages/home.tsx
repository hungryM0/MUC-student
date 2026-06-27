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
  const [accountToDelete, setAccountToDelete] = useState<AccountDto | null>(null);
  const [accountForm, setAccountForm] = useState<AccountFormState | null>(null);
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
      if (nextSnapshot.loginState.message) {
        setErrorText(nextSnapshot.loginState.message);
      } else {
        setAccountForm(null);
      }
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
    setDeletingAccountId(account.id);
    setErrorText("");
    try {
      setSnapshot(await deleteAccount(account.id));
      setAccountToDelete(null);
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setDeletingAccountId("");
    }
  }

  return (
    <WindowFrame
      titleBar={
        <MainTitleBar
          onOpenSettings={() => setSettingsOpen(true)}
          onOpenAbout={() => setAboutOpen(true)}
        />
      }
      contentClassName="flex flex-1 overflow-hidden"
    >
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <main className="min-w-0 flex-1 overflow-y-auto p-6">
          <div className="mx-auto flex max-w-5xl flex-col gap-6">
            {/* 顶栏 Header */}
            <header className="flex items-center justify-between border-b border-border/40 pb-4">
              <div className="flex items-center gap-4">
                <h1 className="text-xl font-bold tracking-tight text-foreground">
                  MUC 校园网拼车
                </h1>
                <div className="bg-border/60 h-4 w-px hidden sm:block" />
              </div>
              <span className="text-muted-foreground hidden font-mono text-xs sm:inline">
                {snapshot?.network.ip && snapshot.network.ip !== "unknown"
                  ? snapshot.network.ip
                  : "IP 未识别"}
              </span>
            </header>

            {errorText && (
              <div className="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border px-4 py-3 text-sm flex items-center gap-2">
                <span className="h-1.5 w-1.5 rounded-full bg-destructive" />
                {errorText}
              </div>
            )}

            <div className="flex flex-col gap-6 md:flex-row">
              {/* 左侧：账号池 */}
              <div className="flex-1 min-w-0">
                <Card className="border-border/70 bg-background/90 flex flex-col rounded-xl shadow-sm backdrop-blur-sm overflow-hidden h-full">
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
                        onClick={() => runSnapshotAction("refresh", refreshDashboard)}
                        disabled={isBusy}
                        className="h-8 gap-1.5"
                      >
                        <RefreshCw
                          className={cn(
                            "h-3.5 w-3.5",
                            runningAction === "refresh" && "animate-spin",
                          )}
                        />
                        同步数据
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent className="p-6">
                    <div className="max-h-[calc(100vh-240px)] min-h-[300px] overflow-y-auto pr-1">
                      <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
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
              <div className="w-full md:w-80 shrink-0 flex flex-col gap-6">
                <Card className="border-border/70 bg-background/90 rounded-xl shadow-sm backdrop-blur-sm">
                  <CardHeader className="pb-3 border-b border-border/40">
                    <CardTitle className="flex items-center justify-between text-base font-semibold">
                      <div className="flex items-center gap-2">
                        <CircleGauge className="text-emerald-500 h-4.5 w-4.5" />
                        状态概览
                      </div>
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="pt-4 space-y-5">
                    {/* 流量池部分 */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-medium text-muted-foreground">流量池已用</span>
                        <span className="text-sm font-bold text-emerald-500">{safeProgress}%</span>
                      </div>
                      <div className="bg-muted h-2 rounded-full overflow-hidden">
                        <div
                          className="h-full rounded-full bg-emerald-500 transition-[width] duration-500"
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
                          {snapshot?.poolQuota.includedPackageText || "套餐信息为空"}
                        </div>
                      </div>
                    </div>

                    <div className="border-t border-border/40 my-3" />

                    {/* 在线状态部分 */}
                    <div className="space-y-3.5">
                      <div className="flex items-center justify-between gap-3">
                        <span className="text-muted-foreground text-xs">当前在线</span>
                        <span className="max-w-[70%] truncate text-xs font-medium">
                          {snapshot?.currentOnlineAccountId
                            ? snapshot?.accounts.find(
                                (account) =>
                                  account.id === snapshot.currentOnlineAccountId,
                              )?.remarkName || "未知账号"
                            : "无在线设备"}
                        </span>
                      </div>

                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground text-xs">最近登录</span>
                        <span className="text-xs font-mono text-muted-foreground">
                          {formatLocalLoginTime(snapshot?.loginState.lastLoginTime)}
                        </span>
                      </div>

                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground text-xs">已配置账号</span>
                        <span className="text-xs font-medium">
                          {snapshot?.accounts.length || 0} 个拼车账号
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
                          onClick={() => runSnapshotAction("logout", logoutLocalDevice)}
                          className="w-full h-8 gap-1.5 text-xs text-muted-foreground hover:bg-destructive/10 hover:text-destructive hover:border-destructive/30 transition-all border border-border/30"
                        >
                          <Power className="h-3 w-3" />
                          断开本机连接
                        </Button>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </div>
            </div>
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
      <SettingsDialog open={settingsOpen} onClose={() => setSettingsOpen(false)} />
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
  if (snapshot.statusText === "查询中..." || snapshot.statusText === "查询失败") {
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
  const progress = Math.round(snapshot?.progressPercent ?? 0);
  const accountState = account.isCurrentOnline ? "online" : "idle";

  return (
    <div
      className={cn(
        "grid min-h-20 grid-cols-[1fr_148px] gap-4 rounded-lg border p-4 text-left transition-all duration-300 ease-out",
        accountState === "online"
          ? "border-emerald-500/40 bg-emerald-500/10 shadow-[0_0_12px_rgba(16,185,129,0.15)] ring-1 ring-emerald-500/20 account-card-online"
          : "border-border hover:bg-muted/50 hover:border-muted-foreground/20 hover:shadow-xs",
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
            className={cn(
              "h-8 w-full transition-all duration-300 relative overflow-hidden",
              (loggingIn || selecting) && "bg-emerald-600 hover:bg-emerald-600 text-white"
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
    </div>
  );
}

function AccountStateBadge({
  state,
}: {
  state: "online" | "idle";
}) {
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
                const updated = { ...localForm, remarkName: event.target.value };
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
          确定要删除账号“<span className="font-semibold text-foreground">{account.remarkName}</span>”吗？
        </div>

        <DialogFooter>
          <Button variant="outline" size="sm" onClick={onClose} disabled={deleting}>
            取消
          </Button>
          <Button variant="destructive" size="sm" onClick={onConfirm} disabled={deleting}>
            {deleting ? "删除中" : "确认删除"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
