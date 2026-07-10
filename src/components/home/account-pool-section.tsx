import { useState, useEffect } from "react";
import {
  Activity,
  CheckCircle2,
  LogIn,
  Pencil,
  Plus,
  RefreshCw,
  Trash2,
  ArrowDownWideNarrow,
  Clock,
  ArrowDownAZ,
  ChevronDown,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  FREE_PRODUCT_QUOTA_GB,
  type AccountDto,
  type AppSnapshotDto,
} from "@/lib/muc";
import { cn, isAndroid } from "@/lib/utils";
import type { AccountAction, RunningAction } from "./types";
import {
  buildAccountUsage,
  formatSnapshotSyncText,
  formatTrafficAmount,
  parseTrafficValue,
  trafficProgressClasses,
} from "./utils";

interface AccountPoolSectionProps {
  snapshot: AppSnapshotDto | null;
  loading: boolean;
  isBusy: boolean;
  selectingId: string;
  loginAccountId: string;
  deletingAccountId: string;
  runningAction: RunningAction | null;
  onOpenAddAccount: () => void;
  onRefresh: () => void;
  onEditAccount: AccountAction;
  onDeleteAccount: AccountAction;
  onLoginAccount: AccountAction;
}

export type SortOption = "remaining" | "recent" | "name";

export function sortAccounts(
  rawAccounts: AccountDto[],
  sortBy: SortOption,
  lastSelectedMap: Record<string, number>,
  selectedAccountId?: string,
) {
  return [...rawAccounts].sort((a, b) => {
    const unlimitedA = a.snapshot?.isUnlimitedPlan ?? false;
    const unlimitedB = b.snapshot?.isUnlimitedPlan ?? false;
    if (unlimitedA !== unlimitedB) {
      return unlimitedA ? -1 : 1;
    }

    if (sortBy === "remaining") {
      const getRemaining = (account: AccountDto) => {
        if (!account.snapshot) return -1;
        const { freeUsed, freeQuota } = buildAccountUsage(account);
        const packageAvailable = parseTrafficValue(
          account.snapshot.packageAvailableText,
        );
        const freeRemaining = Math.max(0, freeQuota - freeUsed);
        return freeRemaining + packageAvailable;
      };
      const remainingA = getRemaining(a);
      const remainingB = getRemaining(b);
      if (remainingA !== remainingB) {
        return remainingB - remainingA;
      }
    } else if (sortBy === "recent") {
      const timeA = lastSelectedMap[a.id] || 0;
      const timeB = lastSelectedMap[b.id] || 0;
      if (timeA !== timeB) {
        return timeB - timeA;
      }
    } else {
      const nameA = a.remarkName || "";
      const nameB = b.remarkName || "";
      const comparison = nameA.localeCompare(nameB, "zh-CN");
      if (comparison !== 0) {
        return comparison;
      }
    }

    if (a.isCurrentOnline !== b.isCurrentOnline) {
      return a.isCurrentOnline ? -1 : 1;
    }
    if (a.id === selectedAccountId) return -1;
    if (b.id === selectedAccountId) return 1;
    return a.id.localeCompare(b.id);
  });
}

export function AccountPoolSection({
  snapshot,
  loading,
  isBusy,
  selectingId,
  loginAccountId,
  deletingAccountId,
  runningAction,
  onOpenAddAccount,
  onRefresh,
  onEditAccount,
  onDeleteAccount,
  onLoginAccount,
}: AccountPoolSectionProps) {
  const [sortBy, setSortBy] = useState<SortOption>(() => {
    try {
      const saved = localStorage.getItem("muc_account_sort_by");
      return (saved as SortOption) || "remaining";
    } catch {
      return "remaining";
    }
  });

  useEffect(() => {
    localStorage.setItem("muc_account_sort_by", sortBy);
  }, [sortBy]);

  const [lastSelectedMap, setLastSelectedMap] = useState<
    Record<string, number>
  >(() => {
    try {
      const saved = localStorage.getItem("muc_last_selected_accounts");
      return saved ? JSON.parse(saved) : {};
    } catch {
      return {};
    }
  });

  useEffect(() => {
    if (snapshot?.selectedAccountId) {
      setLastSelectedMap((prev) => {
        const next = { ...prev, [snapshot.selectedAccountId]: Date.now() };
        localStorage.setItem(
          "muc_last_selected_accounts",
          JSON.stringify(next),
        );
        return next;
      });
    }
  }, [snapshot?.selectedAccountId]);

  const [displayAccounts, setDisplayAccounts] = useState<AccountDto[]>(() => {
    return sortAccounts(
      snapshot?.accounts || [],
      sortBy,
      lastSelectedMap,
      snapshot?.selectedAccountId,
    );
  });

  useEffect(() => {
    const sorted = sortAccounts(
      snapshot?.accounts || [],
      sortBy,
      lastSelectedMap,
      snapshot?.selectedAccountId,
    );

    const currentIds = displayAccounts.map((a) => a.id).join(",");
    const newIds = sorted.map((a) => a.id).join(",");
    const hasOrderChanged = currentIds !== newIds;

    if (hasOrderChanged && document.startViewTransition && !isAndroid()) {
      const transition = document.startViewTransition(() => {
        setDisplayAccounts(sorted);
      });
      transition.ready.catch(() => undefined);
    } else {
      setDisplayAccounts(sorted);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    snapshot?.accounts,
    snapshot?.selectedAccountId,
    sortBy,
    lastSelectedMap,
  ]);

  return (
    <Card className="border-border bg-background/95 flex h-full flex-col overflow-hidden rounded-xl backdrop-blur-sm">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 border-b border-border/40 pb-4">
        <div className="space-y-1">
          <CardTitle className="flex items-center gap-2 text-lg font-semibold">
            <Activity className="h-5 w-5 text-amber-500" />
            账号池
          </CardTitle>
          <p className="text-xs text-muted-foreground">
            管理和切换校园网计费账号
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={onOpenAddAccount}
            disabled={isBusy}
            className="h-8 w-8 px-0 md:w-auto md:px-3 md:gap-1.5"
          >
            <Plus className="h-4 w-4" />
            <span className="hidden md:inline">添加账号</span>
          </Button>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="h-8 gap-1.5 px-2 text-xs"
                disabled={isBusy}
              >
                {sortBy === "remaining" && (
                  <ArrowDownWideNarrow className="h-3.5 w-3.5 text-amber-500" />
                )}
                {sortBy === "recent" && (
                  <Clock className="h-3.5 w-3.5 text-blue-500" />
                )}
                {sortBy === "name" && (
                  <ArrowDownAZ className="h-3.5 w-3.5 text-emerald-500" />
                )}
                <span className="hidden md:inline">
                  {sortBy === "remaining" && "剩余量"}
                  {sortBy === "recent" && "最近"}
                  {sortBy === "name" && "备注"}
                </span>
                <ChevronDown className="h-3 w-3 opacity-50" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-44">
              <DropdownMenuRadioGroup
                value={sortBy}
                onValueChange={(val) => setSortBy(val as SortOption)}
              >
                <DropdownMenuRadioItem value="remaining" className="gap-2">
                  <ArrowDownWideNarrow className="h-3.5 w-3.5 text-amber-500" />
                  <span className="whitespace-nowrap">剩余量</span>
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="recent" className="gap-2">
                  <Clock className="h-3.5 w-3.5 text-blue-500" />
                  <span className="whitespace-nowrap">最近选择</span>
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="name" className="gap-2">
                  <ArrowDownAZ className="h-3.5 w-3.5 text-emerald-500" />
                  <span className="whitespace-nowrap">备注名 A-Z</span>
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
            </DropdownMenuContent>
          </DropdownMenu>

          <Button
            variant="outline"
            size="sm"
            onClick={onRefresh}
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
        <div
          className={cn(
            "min-h-[300px] pr-1",
            isAndroid()
              ? "max-h-none"
              : "max-h-[calc(100vh-240px)] overflow-y-auto",
          )}
        >
          <div className="grid grid-cols-1 gap-3 md:gap-4 xl:grid-cols-2 2xl:grid-cols-3">
            {displayAccounts.length ? (
              displayAccounts.map((account) => (
                <AccountCard
                  key={account.id}
                  account={account}
                  selecting={selectingId === account.id}
                  loggingIn={loginAccountId === account.id}
                  disabled={isBusy}
                  deleting={deletingAccountId === account.id}
                  onEdit={() => onEditAccount(account)}
                  onDelete={() => onDeleteAccount(account)}
                  onLogin={() => onLoginAccount(account)}
                />
              ))
            ) : (
              <div className="col-span-full flex h-48 flex-col items-center justify-center gap-2 rounded-xl border border-dashed border-border/80 bg-muted/10 text-sm text-muted-foreground">
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
  );
}

interface AccountCardProps {
  account: AccountDto;
  selecting: boolean;
  loggingIn: boolean;
  disabled: boolean;
  deleting: boolean;
  onEdit: () => void;
  onDelete: () => void;
  onLogin: () => void;
}

function AccountCard({
  account,
  selecting,
  loggingIn,
  disabled,
  deleting,
  onEdit,
  onDelete,
  onLogin,
}: AccountCardProps) {
  const snapshot = account.snapshot;
  const {
    freeUsed,
    totalUsed,
    totalQuota,
    packageTotal,
    packageUsed,
    freeProgress,
    packageProgress,
    totalProgress,
    isUnlimitedPlan,
  } = buildAccountUsage(account);
  const accountState = account.isCurrentOnline ? "online" : "idle";

  return (
    <div
      style={
        {
          viewTransitionName: `account-card-${account.id}`,
        } as React.CSSProperties & { viewTransitionName: string }
      }
      className={cn(
        "grid min-h-20 gap-y-3 rounded-lg border text-left transition-all duration-300 ease-out animate-slide-in-up",
        isAndroid()
          ? "grid-cols-[1fr_132px] gap-x-2.5 p-3"
          : "grid-cols-[1fr_148px] gap-x-4 p-4",
        isUnlimitedPlan
          ? "unlimited-plan-card"
          : accountState === "online"
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
        <div className="truncate text-sm text-muted-foreground">
          {account.username}
        </div>
        <div
          className={cn(
            "flex flex-wrap gap-y-1 text-muted-foreground",
            isAndroid() ? "gap-x-2 text-[10px]" : "gap-x-4 text-xs",
          )}
        >
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
              isUnlimitedPlan
                ? "unlimited-plan-text"
                : trafficProgressClasses(Math.round(totalProgress)).text,
            )}
          >
            {isUnlimitedPlan ? "不限流量" : `${Math.round(totalProgress)}%`}
          </div>
          <div
            className={cn(
              "mt-0.5 truncate text-muted-foreground",
              isAndroid() ? "text-[10px]" : "text-xs",
            )}
          >
            {isUnlimitedPlan
              ? formatTrafficAmount(totalUsed)
              : `${formatTrafficAmount(totalUsed)} / ${formatTrafficAmount(totalQuota)}`}
          </div>
        </div>

        <div className="grid w-full grid-cols-[1fr_32px_32px] gap-1.5">
          <Button
            type="button"
            size="sm"
            disabled={disabled}
            onClick={onLogin}
            className={cn(
              "relative h-8 w-full overflow-hidden transition-all duration-300",
              isAndroid() ? "px-1 text-xs" : "px-3",
              (loggingIn || selecting) &&
                "bg-emerald-600 text-white hover:bg-emerald-600",
            )}
          >
            <span className="flex items-center justify-center gap-1 transition-all duration-300 whitespace-nowrap">
              {loggingIn || selecting ? (
                <RefreshCw className="h-3.5 w-3.5 shrink-0 animate-spin" />
              ) : (
                <LogIn className="h-3.5 w-3.5 shrink-0" />
              )}
              <span className="shrink-0">
                {loggingIn || selecting ? "登录中" : "登录"}
              </span>
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
            className="h-8 w-8 transition-transform duration-200 hover:scale-105 hover:text-destructive active:scale-95"
          >
            <Trash2
              className={cn("h-3.5 w-3.5", deleting && "animate-pulse")}
            />
          </Button>
        </div>
      </div>

      {!isUnlimitedPlan && (
        <div className="col-span-2 mt-1">
          <div className="grid gap-2">
            <div className="grid gap-1">
              <div className="flex justify-between text-[11px] text-muted-foreground">
                <span>免费包</span>
                <span>{Math.round(freeProgress)}%</span>
              </div>
              <div
                className={cn(
                  "h-1.5 w-full overflow-hidden rounded-full",
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
                <span>{formatTrafficAmount(freeUsed)}</span>
                <span>{formatTrafficAmount(FREE_PRODUCT_QUOTA_GB)}</span>
              </div>
            </div>

            {packageTotal > 0 && (
              <div className="grid gap-1">
                <div className="flex justify-between text-[11px] text-muted-foreground">
                  <span>套餐流量</span>
                  <span>{Math.round(packageProgress)}%</span>
                </div>
                <div className="bg-muted h-1.5 overflow-hidden rounded-full">
                  <div
                    className="h-full rounded-full bg-sky-500 transition-[width] duration-500 ease-out"
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
      )}
    </div>
  );
}

function AccountStateBadge({ state }: { state: "online" | "idle" }) {
  const isOnline = state === "online";

  return (
    <span
      className={cn(
        "inline-flex origin-left shrink-0 transform items-center gap-1 rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs text-emerald-600 transition-all duration-300",
        isOnline
          ? "max-w-[100px] translate-x-0 scale-100 opacity-100"
          : "pointer-events-none max-w-0 translate-x-[-10px] scale-90 overflow-hidden opacity-0",
      )}
    >
      <CheckCircle2 className="h-3 w-3 shrink-0" />
      <span className="shrink-0">在线</span>
    </span>
  );
}
