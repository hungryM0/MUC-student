import { Body1, Caption1, Card, Text } from '@fluentui/react-components';
import type { PoolQuotaDto } from '$lib/types/app';
import { progressText } from '$lib/features/shared/format';
import { ProgressBar } from '$lib/features/shared/progress';
import { useShellStyles } from '$lib/features/shared/layout';

type Props = {
  quota: PoolQuotaDto | null;
  loading?: boolean;
};

export function QuotaSummaryCard({ quota, loading = false }: Props) {
  const styles = useShellStyles();

  return (
    <Card appearance="filled" className={`${styles.card} ${styles.lightSafeCard}`} style={{ padding: 24 }}>
      <div className={styles.splitRow}>
        <Text weight="semibold" size={500}>
          号池总配额
        </Text>
        <Caption1 className={styles.muted}>{loading ? '加载中...' : progressText(quota?.progressPercent)}</Caption1>
      </div>
      <ProgressBar value={quota?.progressPercent} loading={loading} />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, minmax(0, 1fr))', gap: 12 }}>
        <Body1>
          <span className={styles.muted}>已用流量：</span>
          {loading ? '-' : quota?.usedTrafficText || '-'}
        </Body1>
        <Body1>
          <span className={styles.muted}>号池总流量：</span>
          {loading ? '正在加载...' : quota?.productBalanceText || '-'}
          {!loading && quota?.includedPackageText ? <span className={styles.success}> [{quota.includedPackageText.trim()}]</span> : null}
        </Body1>
      </div>
    </Card>
  );
}
