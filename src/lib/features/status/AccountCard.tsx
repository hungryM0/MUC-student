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
    <Card className={cn(
      'grid grid-cols-[72px_1fr_auto] gap-4 p-4 border border-border/80 shadow-[0_1px_3px_rgba(0,0,0,0.02)] transition-all duration-200 hover:bg-muted/30',
      selected && 'border-primary ring-1 ring-primary/60 bg-primary/[0.02]'
    )}>
      <div className="flex items-center justify-center">
        <TrafficRing value={account.snapshot?.progressPercent} />
      </div>
      <div className="min-w-0 space-y-1.5">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="truncate text-sm font-semibold text-foreground tracking-tight">
            {account.remarkName || account.username}
          </h3>
          {account.isCurrentOnline && (
            <Badge className="bg-primary/10 text-primary border-none text-[10px] font-medium h-4 px-1.5 rounded-full hover:bg-primary/10">
              本机在线
            </Badge>
          )}
        </div>
        <div className="grid gap-x-6 gap-y-1 text-xs text-muted-foreground sm:grid-cols-2">
          <span className="truncate">账号：{account.username}</span>
          <span>设备数：{(account.snapshot?.onlineDeviceCountText || '').trim() || '-'}</span>
          <span>已用：{account.snapshot?.usedTrafficText || '-'}</span>
          <span className="flex items-center flex-wrap gap-1">
            总额：{account.snapshot?.productBalanceText || '-'}
            {account.snapshot?.includedPackageText && (
              <span className="text-[9px] text-emerald-500 font-medium bg-emerald-500/10 px-1 rounded">
                {account.snapshot.includedPackageText.trim()}
              </span>
            )}
          </span>
          <span className="sm:col-span-2 text-[10px] opacity-80 mt-0.5">
            更新：{formatDateTime(account.snapshot?.queriedAt)}
          </span>
        </div>
      </div>
      <div className="flex items-center gap-1">
        {account.canLogoutLocalDevice && (
          <Button
            size="icon"
            variant="ghost"
            className="h-8 w-8 rounded-md text-destructive hover:bg-destructive/10 hover:text-destructive cursor-pointer shrink-0"
            disabled={!actionEnabled}
            onClick={() => onLogout(account.id)}
            aria-label="下线本机"
          >
            <LogOut className="size-4" />
          </Button>
        )}
        <Button
          size="icon"
          variant="ghost"
          className="h-8 w-8 rounded-md text-muted-foreground hover:bg-muted/80 hover:text-foreground cursor-pointer shrink-0"
          disabled={!actionEnabled}
          onClick={() => onEdit(account.id)}
          aria-label="编辑账号"
        >
          <Edit3 className="size-4" />
        </Button>
        <Button
          size="icon"
          variant="ghost"
          className="h-8 w-8 rounded-md text-muted-foreground hover:bg-destructive/10 hover:text-destructive cursor-pointer shrink-0"
          disabled={!actionEnabled}
          onClick={() => onDelete(account.id)}
          aria-label="删除账号"
        >
          <Trash2 className="size-4" />
        </Button>
      </div>
    </Card>
  );
}
