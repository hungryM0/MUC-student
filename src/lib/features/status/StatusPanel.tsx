import { Button, Card, Select, Text } from '@fluentui/react-components';
import { AddRegular, ArrowClockwiseRegular, ChevronRightRegular } from '@fluentui/react-icons';
import { useMemo, useState } from 'react';
import type { AccountDto, AppSnapshotDto, UiState } from '$lib/types/app';
import { iconSize, useShellStyles } from '$lib/features/shared/layout';
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
  const styles = useShellStyles();
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
    <section className={styles.stack} style={{ flex: 1 }}>
      <div className={styles.toolbar}>
        <Text className={styles.muted} style={{ marginRight: 'auto' }}>
          已添加 {snapshot.accounts.length} 个账号
        </Text>
        <Select value={sortMode} onChange={(event) => onSortMode(event.target.value as UiState['sortMode'])}>
          <option value="default">默认排序</option>
          <option value="remainingDesc">剩余量高到低</option>
          <option value="nameAsc">姓名 A-Z</option>
        </Select>
        <Button appearance="subtle" icon={<ArrowClockwiseRegular style={iconSize} />} disabled={busy || snapshot.refreshState.running} onClick={onRefresh}>
          刷新
        </Button>
        <Button appearance="primary" icon={<AddRegular style={iconSize} />} disabled={busy} onClick={onAdd}>
          添加账号
        </Button>
      </div>

      <QuotaSummaryCard quota={snapshot.poolQuota} loading={snapshot.refreshState.running} />

      <div className={styles.scrollArea}>
        {snapshot.accounts.length === 0 ? (
          <Card appearance="filled" className={styles.compactCard} style={{ textAlign: 'center' }}>
            <Text className={styles.muted}>还没有账号。</Text>
          </Card>
        ) : (
          <div className={styles.stack}>
            {activeAccounts.map((account) => (
              <AccountCard
                key={account.id}
                account={account}
                selected={account.id === snapshot.selectedAccountId}
                actionEnabled={actionEnabled}
                onEdit={onEdit}
                onDelete={onDelete}
                onLogout={onLogout}
              />
            ))}

            {exhaustedAccounts.length > 0 && (
              <div className={styles.stack}>
                <Button
                  appearance="subtle"
                  icon={<ChevronRightRegular style={{ ...iconSize, transform: collapsed ? 'rotate(0deg)' : 'rotate(90deg)' }} />}
                  onClick={() => setCollapsed((value) => !value)}
                >
                  已用尽账号（{exhaustedAccounts.length}）
                </Button>
                {!collapsed &&
                  exhaustedAccounts.map((account) => (
                    <AccountCard
                      key={account.id}
                      account={account}
                      selected={account.id === snapshot.selectedAccountId}
                      actionEnabled={actionEnabled}
                      onEdit={onEdit}
                      onDelete={onDelete}
                      onLogout={onLogout}
                    />
                  ))}
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
