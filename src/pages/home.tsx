import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Activity,
  CheckCircle2,
  CircleGauge,
  LogIn,
  Power,
  RefreshCw,
  Router,
  UserRound,
  Wifi,
  type LucideIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { WindowFrame } from "@/components/window-frame";
import { MainTitleBar } from "@/components/main-title-bar";
import {
  type AccountDto,
  type AppSnapshotDto,
  bootstrapApp,
  loginSelectedAccount,
  logoutLocalDevice,
  readErrorMessage,
  refreshDashboard,
  selectAccount,
} from "@/lib/muc";
import { cn } from "@/lib/utils";

type RunningAction = "login" | "refresh" | "logout";

export default function HomePage() {
  const [snapshot, setSnapshot] = useState<AppSnapshotDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [errorText, setErrorText] = useState("");
  const [selectingId, setSelectingId] = useState("");
  const [loginAccountId, setLoginAccountId] = useState("");
  const [runningAction, setRunningAction] = useState<RunningAction | null>(
    null,
  );

  useEffect(() => {
    const initTrayMenu = async () => {
      try {
        await invoke("update_tray_menu", {
          showText: "显示窗口",
          quitText: "退出",
        });
      } catch (error) {
        console.error("Failed to initialize tray menu:", error);
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

  const selectedAccount = useMemo(
    () =>
      snapshot?.accounts.find(
        (account) => account.id === snapshot.selectedAccountId,
      ),
    [snapshot],
  );

  const progressPercent = Math.round(
    (snapshot?.poolQuota.progressPercent ?? 0) * 100,
  );
  const safeProgress = Math.min(100, Math.max(0, progressPercent));
  const isBusy =
    loading ||
    !!selectingId ||
    !!runningAction ||
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

  return (
    <WindowFrame
      titleBar={<MainTitleBar />}
      contentClassName="flex flex-1 overflow-hidden bg-[radial-gradient(circle_at_top_left,rgba(14,165,233,0.14),transparent_34%),linear-gradient(135deg,rgba(16,185,129,0.08),transparent_42%)]"
    >
      <div className="grid min-h-0 flex-1 grid-cols-[240px_1fr] overflow-hidden">
        <aside className="border-border/70 bg-background/80 flex flex-col border-r px-5 py-5">
          <div className="space-y-1">
            <div className="text-muted-foreground text-xs font-medium tracking-[0.18em] uppercase">
              MUC
            </div>
            <div className="text-2xl font-semibold tracking-normal">
              校园网拼车
            </div>
          </div>

          <div className="mt-8 space-y-2">
            <NavItem icon={CircleGauge} active label="总览" />
            <NavItem icon={UserRound} label="账号" />
            <NavItem icon={Router} label="设备" />
          </div>

          <div className="mt-auto space-y-3">
            <StatusPill
              tone={snapshot?.network.isOnline ? "green" : "amber"}
              label={snapshot?.network.statusText || "IP 未识别"}
            />
            <div className="text-muted-foreground truncate text-xs">
              {snapshot?.network.ip && snapshot.network.ip !== "unknown"
                ? snapshot.network.ip
                : "IP 未识别"}
            </div>
          </div>
        </aside>

        <main className="min-w-0 overflow-auto p-6">
          <div className="mx-auto flex max-w-6xl flex-col gap-5">
            <section className="grid grid-cols-[1.25fr_0.75fr] gap-5">
              <Card className="border-border/70 bg-background/90 rounded-lg shadow-none">
                <CardHeader className="pb-3">
                  <CardTitle className="flex items-center gap-2 text-base">
                    <CircleGauge className="text-emerald-500 h-4 w-4" />
                    流量池
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-5">
                  <div className="flex items-end justify-between gap-4">
                    <div className="min-w-0">
                      <div className="truncate text-3xl font-semibold tracking-normal">
                        {snapshot?.poolQuota.usedTrafficText ?? "-"}
                      </div>
                      <div className="text-muted-foreground mt-1 truncate text-sm">
                        {snapshot?.poolQuota.productBalanceText ?? "-"}
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-2xl font-semibold">
                        {safeProgress}%
                      </div>
                      <div className="text-muted-foreground text-xs">已用</div>
                    </div>
                  </div>
                  <div className="bg-muted h-2 overflow-hidden rounded-full">
                    <div
                      className="h-full rounded-full bg-emerald-500 transition-[width]"
                      style={{ width: `${safeProgress}%` }}
                    />
                  </div>
                  <div className="text-muted-foreground truncate text-sm">
                    {snapshot?.poolQuota.includedPackageText || "套餐信息为空"}
                  </div>
                </CardContent>
              </Card>

              <Card className="border-border/70 bg-background/90 rounded-lg shadow-none">
                <CardHeader className="pb-3">
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Wifi className="text-sky-500 h-4 w-4" />
                    当前状态
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Metric
                    label="已选账号"
                    value={selectedAccount?.remarkName || "-"}
                  />
                  <Metric
                    label="在线账号"
                    value={
                      snapshot?.accounts.find(
                        (account) =>
                          account.id === snapshot.currentOnlineAccountId,
                      )?.remarkName || "-"
                    }
                  />
                  <Metric
                    label="最近登录"
                    value={
                      snapshot?.loginState.resultText ||
                      snapshot?.loginState.message ||
                      "-"
                    }
                  />
                </CardContent>
              </Card>
            </section>

            {errorText && (
              <div className="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border px-4 py-3 text-sm">
                {errorText}
              </div>
            )}

            <section className="grid min-h-[430px] grid-cols-[minmax(0,1fr)_280px] gap-5">
              <Card className="border-border/70 bg-background/90 flex min-h-0 flex-col rounded-lg shadow-none">
                <CardHeader className="flex-row items-center justify-between space-y-0 pb-3">
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Activity className="text-amber-500 h-4 w-4" />
                    账号池
                  </CardTitle>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={loadSnapshot}
                    disabled={isBusy}
                    className="h-8 gap-2"
                  >
                    <RefreshCw
                      className={cn("h-3.5 w-3.5", loading && "animate-spin")}
                    />
                    同步
                  </Button>
                </CardHeader>
                <CardContent className="min-h-0 flex-1">
                  <div className="max-h-[calc(100vh-430px)] min-h-[280px] overflow-y-auto pr-1">
                    <div className="grid gap-3">
                      {snapshot?.accounts.length ? (
                        snapshot.accounts.map((account) => (
                          <AccountRow
                            key={account.id}
                            account={account}
                            selected={account.id === snapshot.selectedAccountId}
                            selecting={selectingId === account.id}
                            loggingIn={loginAccountId === account.id}
                            disabled={isBusy}
                            onLogin={() => handleLoginAccount(account)}
                          />
                        ))
                      ) : (
                        <div className="text-muted-foreground flex h-40 items-center justify-center rounded-lg border border-dashed text-sm">
                          {loading ? "读取中" : "暂无账号"}
                        </div>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="border-border/70 bg-background/90 rounded-lg shadow-none">
                <CardHeader className="pb-3">
                  <CardTitle className="text-base">全局动作</CardTitle>
                </CardHeader>
                <CardContent className="grid gap-2">
                  <Button
                    className="justify-start gap-2"
                    disabled={isBusy || !snapshot?.accounts.length}
                    onClick={() =>
                      runSnapshotAction("refresh", refreshDashboard)
                    }
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
                    className="justify-start gap-2"
                    variant="outline"
                    disabled={isBusy || !canLogoutLocalDevice}
                    onClick={() =>
                      runSnapshotAction("logout", logoutLocalDevice)
                    }
                  >
                    <Power
                      className={cn(
                        "h-4 w-4",
                        runningAction === "logout" && "animate-pulse",
                      )}
                    />
                    本机下线
                  </Button>
                </CardContent>
              </Card>
            </section>
          </div>
        </main>
      </div>
    </WindowFrame>
  );
}

function NavItem({
  icon: Icon,
  label,
  active = false,
}: {
  icon: LucideIcon;
  label: string;
  active?: boolean;
}) {
  return (
    <button
      className={cn(
        "flex h-9 w-full items-center gap-2 rounded-md px-3 text-left text-sm",
        active
          ? "bg-foreground text-background"
          : "text-muted-foreground hover:bg-muted",
      )}
    >
      <Icon className="h-4 w-4" />
      {label}
    </button>
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

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-muted-foreground text-sm">{label}</span>
      <span className="truncate text-sm font-medium">{value}</span>
    </div>
  );
}

function AccountRow({
  account,
  selected,
  selecting,
  loggingIn,
  disabled,
  onLogin,
}: {
  account: AccountDto;
  selected: boolean;
  selecting: boolean;
  loggingIn: boolean;
  disabled: boolean;
  onLogin: () => void;
}) {
  const snapshot = account.snapshot;
  const progress = Math.round((snapshot?.progressPercent ?? 0) * 100);
  const accountState = account.isCurrentOnline
    ? "online"
    : selected
      ? "selected"
      : "idle";

  return (
    <div
      className={cn(
        "grid min-h-20 grid-cols-[1fr_112px] gap-4 rounded-lg border p-4 text-left transition-colors",
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
        <div className="w-full">
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
