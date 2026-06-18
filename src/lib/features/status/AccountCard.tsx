import { Badge, Button, Card, Caption1, makeStyles, Text, tokens } from '@fluentui/react-components';
import { DeleteRegular, EditRegular, LockClosedRegular } from '@fluentui/react-icons';
import type { AccountDto } from '$lib/types/app';
import { formatDateTime } from '$lib/features/shared/format';
import { iconSize, useShellStyles } from '$lib/features/shared/layout';
import { TrafficRing } from '$lib/features/shared/progress';

type Props = {
  account: AccountDto;
  selected?: boolean;
  actionEnabled?: boolean;
  onEdit: (accountId: string) => void;
  onDelete: (accountId: string) => void;
  onLogout: (accountId: string) => void;
};

const useAccountStyles = makeStyles({
  card: {
    display: 'grid',
    gridTemplateColumns: '148px minmax(0, 1fr)',
    gap: '28px',
    padding: '24px 28px',
    minHeight: '188px',
    alignItems: 'center',
    border: `1px solid ${tokens.colorNeutralStroke3}`,
    borderRadius: tokens.borderRadiusXLarge,
    backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground2) 76%, transparent)',
    boxShadow: tokens.shadow4
  },
  selected: {
    border: `1px solid ${tokens.colorBrandStroke1}`,
    boxShadow: tokens.shadow8
  },
  ringColumn: {
    display: 'grid',
    placeItems: 'center',
    minWidth: 0
  },
  details: {
    display: 'grid',
    gridTemplateRows: 'auto auto auto',
    gap: '14px',
    minWidth: 0
  },
  titleRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    minWidth: 0
  },
  title: {
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap'
  },
  metrics: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, minmax(0, 1fr))',
    gap: '8px 32px',
    minWidth: 0
  },
  metric: {
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap'
  },
  actions: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px'
  }
});

export function AccountCard({ account, selected = false, actionEnabled = true, onEdit, onDelete, onLogout }: Props) {
  const shell = useShellStyles();
  const styles = useAccountStyles();

  return (
    <Card appearance="filled" className={`${styles.card} ${selected ? styles.selected : ''}`}>
      <div className={styles.ringColumn}>
        <TrafficRing value={account.snapshot?.progressPercent} />
      </div>

      <div className={styles.details}>
        <div className={styles.titleRow}>
          <Text size={500} weight="semibold" className={styles.title}>
            {account.remarkName || account.username}
          </Text>
          {account.isCurrentOnline && <Badge appearance="tint" color="success">本机在线</Badge>}
        </div>

        <div className={styles.metrics}>
          <Caption1 className={`${shell.muted} ${styles.metric}`}>账号：{account.username}</Caption1>
          <Caption1 className={`${shell.muted} ${styles.metric}`}>设备数：{(account.snapshot?.onlineDeviceCountText || '').trim() || '-'}</Caption1>
          <Caption1 className={`${shell.muted} ${styles.metric}`}>已用流量：{account.snapshot?.usedTrafficText || '-'}</Caption1>
          <Caption1 className={`${shell.muted} ${styles.metric}`}>
            账户总流量：{account.snapshot?.productBalanceText || '-'}
            {account.snapshot?.includedPackageText ? <span className={shell.success}> [{account.snapshot.includedPackageText.trim()}]</span> : null}
          </Caption1>
          <Caption1 className={`${shell.muted} ${styles.metric}`} style={{ gridColumn: '1 / -1' }}>
            更新：{formatDateTime(account.snapshot?.queriedAt)}
          </Caption1>
        </div>

        <div className={styles.actions}>
          {account.canLogoutLocalDevice && (
            <Button appearance="primary" icon={<LockClosedRegular style={iconSize} />} disabled={!actionEnabled} onClick={() => onLogout(account.id)}>
              下线本机
            </Button>
          )}
          <Button appearance="secondary" icon={<EditRegular style={iconSize} />} disabled={!actionEnabled} onClick={() => onEdit(account.id)}>
            编辑
          </Button>
          <Button appearance="secondary" icon={<DeleteRegular style={iconSize} />} disabled={!actionEnabled} onClick={() => onDelete(account.id)}>
            删除
          </Button>
        </div>
      </div>
    </Card>
  );
}
