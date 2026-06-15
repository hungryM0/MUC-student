import { Edit3, LogOut, Trash2 } from 'lucide-react';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import { Card } from '$lib/components/ui/card';
import type { AccountDto } from '$lib/types/app';
import { cn } from '$lib/utils';
import { formatDateTime } from '$lib/features/shared/format';
import { TrafficRing } from '$lib/features/shared/progress';

type Props = {
  account: AccountDto;
  selected?: boolean;
  actionEnabled?: boolean;
  onEdit: (accountId: string) => void;
  onDelete: (accountId: string) => void;
  onLogout: (accountId: string) => void;
};

export function AccountCard({ account, selected = false, actionEnabled = true, onEdit, onDelete, onLogout }: Props) {
  return (
    <Card className={cn('grid grid-cols-[84px_1fr_auto] gap-4 p-4', selected && 'border-primary shadow-[0_0_0_1px_hsl(var(--primary))]')}>
      <TrafficRing value={account.snapshot?.progressPercent} />
      <div className="min-w-0 space-y-2">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="truncate text-base font-semibold">{account.remarkName || account.username}</h3>
          {account.isCurrentOnline && <Badge className="bg-primary/15 text-primary hover:bg-primary/15">本机在线</Badge>}
        </div>
        <div className="grid gap-x-5 gap-y-1 text-sm text-muted-foreground sm:grid-cols-2">
          <span>账号：{account.username}</span>
          <span>在线设备：{(account.snapshot?.onlineDeviceCountText || '').trim() || '-'}</span>
          <span>已用流量：{account.snapshot?.usedTrafficText || '-'}</span>
          <span>
            总流量：{account.snapshot?.productBalanceText || '-'}
            {account.snapshot?.includedPackageText ? <b className="ml-1 font-medium text-emerald-500">[{account.snapshot.includedPackageText.trim()}]</b> : null}
          </span>
          <span className="sm:col-span-2">更新时间：{formatDateTime(account.snapshot?.queriedAt)}</span>
        </div>
      </div>
      <div className="flex items-start gap-2">
        {account.canLogoutLocalDevice && (
          <Button size="icon" disabled={!actionEnabled} onClick={() => onLogout(account.id)} aria-label="下线本机">
            <LogOut />
          </Button>
        )}
        <Button size="icon" variant="secondary" disabled={!actionEnabled} onClick={() => onEdit(account.id)} aria-label="编辑账号">
          <Edit3 />
        </Button>
        <Button size="icon" variant="ghost" disabled={!actionEnabled} onClick={() => onDelete(account.id)} aria-label="删除账号">
          <Trash2 />
        </Button>
      </div>
    </Card>
  );
}
