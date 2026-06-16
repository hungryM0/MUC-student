import { LogIn, User, Wifi, ShieldAlert, Monitor, HardDrive, CircleDot } from 'lucide-react';
import { Button } from '$lib/components/ui/button';
import { Card } from '$lib/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '$lib/components/ui/select';
import type { AppSnapshotDto, UiState } from '$lib/types/app';
import { formatDateTime } from '$lib/features/shared/format';
import { ProgressBar } from '$lib/features/shared/progress';
import { StatusPanel } from '$lib/features/status/StatusPanel';
import { cn } from '$lib/utils';

type Props = {
  snapshot: AppSnapshotDto;
  sortMode: UiState['sortMode'];
  busy: boolean;
  onSelectAccount: (accountId: string) => void;
  onLogin: () => void;
  onSortMode: (mode: UiState['sortMode']) => void;
  onAdd: () => void;
  onEdit: (accountId: string) => void;
  onDelete: (accountId: string) => void;
  onLogout: (accountId: string) => void;
  onRefresh: () => void;
};

export function HomePanel({
  snapshot,
  sortMode,
  busy,
  onSelectAccount,
  onLogin,
  onSortMode,
  onAdd,
  onEdit,
  onDelete,
  onLogout,
  onRefresh
}: Props) {
  const activeAccount = snapshot.accounts.find((account) => account.id === snapshot.selectedAccountId) ?? null;
  const isOnline = snapshot.network.isOnline;

  return (
    <section className="panel-in flex min-h-0 flex-1 flex-col gap-4">
      <Card className="relative overflow-hidden p-6 border border-border/80 shadow-sm bg-gradient-to-br from-card to-card/95">
        <div className="absolute top-0 left-0 right-0 h-[3px] bg-gradient-to-r from-primary/80 to-primary" />

        <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-4">
            <div className={cn(
              "flex h-12 w-12 shrink-0 items-center justify-center rounded-lg transition-all duration-300",
              isOnline ? "bg-primary/10 text-primary" : "bg-muted text-muted-foreground"
            )}>
              {isOnline ? <Wifi className="size-6 animate-pulse" /> : <ShieldAlert className="size-6" />}
            </div>
            <div className="min-w-0">
              <h2 className="text-lg font-semibold tracking-tight">
                {isOnline ? (activeAccount?.remarkName || activeAccount?.username || '已登录客户端') : '未登录或已断开'}
              </h2>
              <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs text-muted-foreground">
                <span className="flex items-center gap-1">
                  <span className={cn("h-1.5 w-1.5 rounded-full", isOnline ? "bg-emerald-500" : "bg-rose-500")} />
                  {isOnline ? '校园网已连接' : '网络未认证'}
                </span>
                <span>•</span>
                <span>内网 IP: {snapshot.network.ip || 'unknown'}</span>
                {snapshot.loginState.lastLoginTime && (
                  <>
                    <span>•</span>
                    <span>最近登录: {formatDateTime(snapshot.loginState.lastLoginTime)}</span>
                  </>
                )}
              </div>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <Select
              value={snapshot.selectedAccountId || undefined}
              onValueChange={onSelectAccount}
              disabled={busy || snapshot.loginState.running || snapshot.accounts.length === 0}
            >
              <SelectTrigger className="w-48 h-9 text-xs rounded-md border-border/60 hover:bg-muted/40 cursor-pointer">
                <SelectValue placeholder="选择登录账号" />
              </SelectTrigger>
              <SelectContent>
                {snapshot.accounts.map((account) => (
                  <SelectItem key={account.id} value={account.id} className="text-xs">
                    {account.remarkName || account.username} ({account.username})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Button
              disabled={busy || snapshot.loginState.running || snapshot.accounts.length === 0}
              onClick={onLogin}
              className="h-9 min-w-24 text-xs font-medium rounded-md shadow-sm cursor-pointer"
            >
              <LogIn className="size-3.5" />
              {snapshot.loginState.running ? '登录中...' : '登录认证'}
            </Button>
          </div>
        </div>
      </Card>

      {activeAccount && (
        <Card className="p-6 border border-border/80 shadow-sm bg-card/50">
          <div className="flex items-center justify-between gap-4 pb-3">
            <div>
              <h3 className="text-sm font-semibold tracking-tight flex items-center gap-1.5">
                <CircleDot className="size-3.5 text-primary" />
                当前账号配额
              </h3>
              <p className="text-xs text-muted-foreground mt-0.5">
                {activeAccount.remarkName ? `${activeAccount.remarkName} (${activeAccount.username})` : activeAccount.username}
              </p>
            </div>
            <span className="font-mono text-sm font-semibold text-primary">
              {(activeAccount.snapshot?.progressPercent ?? 0).toFixed(1)}%
            </span>
          </div>

          <ProgressBar value={activeAccount.snapshot?.progressPercent} loading={snapshot.refreshState.running} />

          <div className="grid gap-3 mt-4 sm:grid-cols-3">
            <div className="flex items-center gap-3 rounded-lg border border-border/40 bg-card p-3 shadow-[0_1px_2px_rgba(0,0,0,0.02)]">
              <div className="flex h-8 w-8 items-center justify-center rounded-md bg-muted text-muted-foreground">
                <Monitor className="size-4" />
              </div>
              <div className="min-w-0">
                <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">在线设备</p>
                <p className="text-xs font-semibold text-foreground truncate mt-0.5">
                  {(activeAccount.snapshot?.onlineDeviceCountText || '').trim() || '-'}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-3 rounded-lg border border-border/40 bg-card p-3 shadow-[0_1px_2px_rgba(0,0,0,0.02)]">
              <div className="flex h-8 w-8 items-center justify-center rounded-md bg-muted text-muted-foreground">
                <HardDrive className="size-4" />
              </div>
              <div className="min-w-0">
                <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">已用流量</p>
                <p className="text-xs font-semibold text-foreground truncate mt-0.5">
                  {activeAccount.snapshot?.usedTrafficText || '-'}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-3 rounded-lg border border-border/40 bg-card p-3 shadow-[0_1px_2px_rgba(0,0,0,0.02)]">
              <div className="flex h-8 w-8 items-center justify-center rounded-md bg-muted text-muted-foreground">
                <User className="size-4" />
              </div>
              <div className="min-w-0">
                <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">套餐总计</p>
                <p className="text-xs font-semibold text-foreground truncate mt-0.5 flex items-center flex-wrap gap-1">
                  <span>{activeAccount.snapshot?.productBalanceText || '-'}</span>
                  {activeAccount.snapshot?.includedPackageText && (
                    <span className="text-[10px] text-emerald-500 font-medium bg-emerald-500/10 px-1 rounded">
                      {activeAccount.snapshot.includedPackageText.trim()}
                    </span>
                  )}
                </p>
              </div>
            </div>
          </div>
        </Card>
      )}

      <StatusPanel
        snapshot={snapshot}
        sortMode={sortMode}
        busy={busy}
        onSortMode={onSortMode}
        onAdd={onAdd}
        onEdit={onEdit}
        onDelete={onDelete}
        onLogout={onLogout}
        onRefresh={onRefresh}
      />
    </section>
  );
}
