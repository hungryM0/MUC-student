import { LogIn } from 'lucide-react';
import { Button } from '$lib/components/ui/button';
import { Card } from '$lib/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '$lib/components/ui/select';
import type { AppSnapshotDto } from '$lib/types/app';
import { formatDateTime } from '$lib/features/shared/format';
import { ProgressBar } from '$lib/features/shared/progress';
import { LogPanel } from '$lib/features/logs/LogPanel';

type Props = {
  snapshot: AppSnapshotDto;
  busy: boolean;
  onSelectAccount: (accountId: string) => void;
  onLogin: () => void;
};

export function HomePanel({ snapshot, busy, onSelectAccount, onLogin }: Props) {
  const activeAccount = snapshot.accounts.find((account) => account.id === snapshot.selectedAccountId) ?? null;

  return (
    <section className="panel-in grid min-h-0 flex-1 grid-rows-[auto_auto_1fr] gap-4">
      <Card className="grid gap-4 p-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold">MUC-student</h2>
            <div className="mt-2 grid gap-1 text-sm text-muted-foreground">
              <span>内网 IPv4：{snapshot.network.ip || 'unknown'}</span>
              <span>最近登录：{formatDateTime(snapshot.loginState.lastLoginTime)}</span>
            </div>
          </div>
          <Button disabled={busy || snapshot.loginState.running || snapshot.accounts.length === 0} onClick={onLogin} className="min-w-32">
            <LogIn />
            {snapshot.loginState.running ? '登录中' : snapshot.accounts.length === 0 ? '无账号' : '登录'}
          </Button>
        </div>

        <Select value={snapshot.selectedAccountId || undefined} onValueChange={onSelectAccount} disabled={busy || snapshot.loginState.running || snapshot.accounts.length === 0}>
          <SelectTrigger className="max-w-xl">
            <SelectValue placeholder="请先添加账号" />
          </SelectTrigger>
          <SelectContent>
            {snapshot.accounts.map((account) => (
              <SelectItem key={account.id} value={account.id}>
                {account.remarkName || account.username}（{account.username}）
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </Card>

      {activeAccount && (
        <Card className="grid gap-3 p-5">
          <div className="flex items-center justify-between gap-4">
            <div>
              <h3 className="text-base font-semibold">账号配额</h3>
              <p className="text-sm text-muted-foreground">{activeAccount.remarkName || activeAccount.username}</p>
            </div>
            <span className="font-mono text-sm text-muted-foreground">{activeAccount.snapshot?.progressPercent?.toFixed(1) ?? '--'}%</span>
          </div>
          <ProgressBar value={activeAccount.snapshot?.progressPercent} loading={snapshot.refreshState.running} />
          <div className="grid gap-1 text-sm text-muted-foreground sm:grid-cols-3">
            <span>在线设备：{(activeAccount.snapshot?.onlineDeviceCountText || '').trim() || '-'}</span>
            <span>已用流量：{activeAccount.snapshot?.usedTrafficText || '-'}</span>
            <span>
              总流量：{activeAccount.snapshot?.productBalanceText || '-'}
              {activeAccount.snapshot?.includedPackageText ? <b className="ml-1 font-medium text-emerald-500">[{activeAccount.snapshot.includedPackageText.trim()}]</b> : null}
            </span>
          </div>
        </Card>
      )}

      <LogPanel logs={snapshot.logs} />
    </section>
  );
}
