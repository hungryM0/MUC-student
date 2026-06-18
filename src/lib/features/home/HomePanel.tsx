import { Button, Select, Text } from '@fluentui/react-components';
import { LockClosedRegular } from '@fluentui/react-icons';
import type { AppSnapshotDto, UiState } from '$lib/types/app';
import { formatDateTime } from '$lib/features/shared/format';
import { iconSize, useShellStyles } from '$lib/features/shared/layout';
import { StatusPanel } from '$lib/features/status/StatusPanel';

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
  const styles = useShellStyles();
  const isOnline = snapshot.network.isOnline;

  return (
    <section className={styles.stack} style={{ flex: 1 }}>
      <div className={styles.toolbar}>
        <div className={styles.row} style={{ flex: 1 }}>
          <span
            className={styles.statusDot}
            style={{ backgroundColor: isOnline ? 'var(--colorPaletteGreenForeground1)' : 'var(--colorPaletteRedForeground1)' }}
          />
          <Text>{isOnline ? '校园网已连接' : '网络未认证'}</Text>
          <Text className={styles.muted}>内网 IP: {snapshot.network.ip || 'unknown'}</Text>
          {snapshot.loginState.lastLoginTime && <Text className={styles.muted}>最近登录: {formatDateTime(snapshot.loginState.lastLoginTime)}</Text>}
        </div>

        <Select
          value={snapshot.selectedAccountId || ''}
          disabled={busy || snapshot.loginState.running || snapshot.accounts.length === 0}
          onChange={(event) => onSelectAccount(event.target.value)}
          style={{ minWidth: 220 }}
        >
          <option value="" disabled>
            选择登录账号
          </option>
          {snapshot.accounts.map((account) => (
            <option key={account.id} value={account.id}>
              {account.remarkName || account.username} ({account.username})
            </option>
          ))}
        </Select>

        <Button
          appearance="primary"
          icon={<LockClosedRegular style={iconSize} />}
          disabled={busy || snapshot.loginState.running || snapshot.accounts.length === 0}
          onClick={onLogin}
        >
          {snapshot.loginState.running ? '登录中...' : '登录认证'}
        </Button>
      </div>

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
