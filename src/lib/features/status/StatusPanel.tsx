import { ChevronRight, Plus, RefreshCcw } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Button } from '$lib/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '$lib/components/ui/select';
import type { AccountDto, AppSnapshotDto, UiState } from '$lib/types/app';
import { AccountCard } from './AccountCard';
import { QuotaSummaryCard } from './QuotaSummaryCard';

type Props = {
  snapshot: AppSnapshotDto;
  sortMode: UiState['sortMode'];
  busy: boolean;
  onSortMode: (mode: UiState['sortMode']) => void;
  onAdd: () => void;
  onEdit: (accountId: string) => void;
  onDelete: (accountId: string) => void;
  onLogout: (accountId: string) => void;
  onRefresh: () => void;
};

function remainingSortKey(account: AccountDto) {
  const percent = account.snapshot?.progressPercent;
  if (typeof percent !== 'number') return -1;
  return 100 - percent;
}

export function StatusPanel({ snapshot, sortMode, busy, onSortMode, onAdd, onEdit, onDelete, onLogout, onRefresh }: Props) {
  const [collapsed, setCollapsed] = useState(true);
  const sortedAccounts = useMemo(() => {
    const accounts = [...snapshot.accounts];
    if (sortMode === 'remainingDesc') accounts.sort((a, b) => remainingSortKey(b) - remainingSortKey(a));
    if (sortMode === 'nameAsc') accounts.sort((a, b) => `${a.remarkName}${a.username}`.localeCompare(`${b.remarkName}${b.username}`, 'zh-CN'));
    return accounts;
  }, [snapshot.accounts, sortMode]);

  const activeAccounts = sortedAccounts.filter((account) => account.isCurrentOnline || (account.snapshot?.progressPercent ?? -1) < 100);
  const exhaustedAccounts = sortedAccounts.filter((account) => !account.isCurrentOnline && (account.snapshot?.progressPercent ?? -1) >= 100);
  const actionEnabled = !busy && !snapshot.refreshState.running && !snapshot.loginState.running;

  return (
    <section className="panel-in flex min-h-0 flex-1 flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-xl font-semibold">状态</h2>
        <div className="flex flex-wrap items-center gap-2">
          <Select value={sortMode} onValueChange={(value) => onSortMode(value as UiState['sortMode'])}>
            <SelectTrigger className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="default">默认排序</SelectItem>
              <SelectItem value="remainingDesc">剩余量从高到低</SelectItem>
              <SelectItem value="nameAsc">姓名 A-Z</SelectItem>
            </SelectContent>
          </Select>
          <Button variant="secondary" disabled={busy || snapshot.refreshState.running} onClick={onRefresh}>
            <RefreshCcw />
            刷新
          </Button>
          <Button disabled={busy} onClick={onAdd}>
            <Plus />
            添加
          </Button>
        </div>
      </div>

      <QuotaSummaryCard quota={snapshot.poolQuota} loading={snapshot.refreshState.running} />

      <div className="min-h-0 flex-1 overflow-y-auto pr-1">
        {snapshot.accounts.length === 0 ? (
          <div className="rounded-lg border border-dashed border-border p-8 text-center text-sm text-muted-foreground">还没有账号。</div>
        ) : (
          <div className="space-y-3">
            {activeAccounts.map((account) => (
              <AccountCard key={account.id} account={account} selected={account.id === snapshot.selectedAccountId} actionEnabled={actionEnabled} onEdit={onEdit} onDelete={onDelete} onLogout={onLogout} />
            ))}

            {exhaustedAccounts.length > 0 && (
              <div className="space-y-3">
                <Button variant="ghost" className="px-2" onClick={() => setCollapsed((value) => !value)}>
                  <ChevronRight className={collapsed ? '' : 'rotate-90'} />
                  已用尽账号（{exhaustedAccounts.length}）
                </Button>
                {!collapsed &&
                  exhaustedAccounts.map((account) => (
                    <AccountCard key={account.id} account={account} selected={account.id === snapshot.selectedAccountId} actionEnabled={actionEnabled} onEdit={onEdit} onDelete={onDelete} onLogout={onLogout} />
                  ))}
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
