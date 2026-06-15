import { Card } from '$lib/components/ui/card';
import type { PoolQuotaDto } from '$lib/types/app';
import { progressText } from '$lib/features/shared/format';
import { ProgressBar } from '$lib/features/shared/progress';

type Props = {
  quota: PoolQuotaDto | null;
  loading?: boolean;
};

export function QuotaSummaryCard({ quota, loading = false }: Props) {
  return (
    <Card className="grid gap-3 p-4">
      <div className="flex items-center justify-between gap-4">
        <strong className="text-sm">号池总配额</strong>
        <span className="font-mono text-xs text-muted-foreground">{loading ? '加载中...' : progressText(quota?.progressPercent)}</span>
      </div>
      <ProgressBar value={quota?.progressPercent} loading={loading} />
      <div className="grid gap-1 text-sm text-muted-foreground sm:grid-cols-2">
        <span>已用流量：{loading ? '-' : quota?.usedTrafficText || '-'}</span>
        <span>
          总流量：{loading ? '正在加载...' : quota?.productBalanceText || '-'}
          {!loading && quota?.includedPackageText ? <b className="ml-1 font-medium text-emerald-500">[{quota.includedPackageText.trim()}]</b> : null}
        </span>
      </div>
    </Card>
  );
}
