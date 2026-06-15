import { Badge } from '$lib/components/ui/badge';
import { Card } from '$lib/components/ui/card';
import type { LogItemDto } from '$lib/types/app';
import { cn } from '$lib/utils';
import { formatTime } from '$lib/features/shared/format';

type Props = {
  logs: LogItemDto[];
};

export function LogPanel({ logs }: Props) {
  return (
    <Card className="flex min-h-0 flex-1 flex-col overflow-hidden bg-card/80">
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <h3 className="text-sm font-semibold">系统日志</h3>
        <Badge variant="outline">{logs.length}</Badge>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto p-3">
        {logs.length === 0 ? (
          <div className="grid h-32 place-items-center text-sm text-muted-foreground">暂无日志</div>
        ) : (
          <div className="space-y-2">
            {logs.map((log, index) => (
              <div key={`${log.timestamp}-${index}`} className="grid grid-cols-[74px_48px_1fr] gap-3 rounded-md bg-muted/60 px-3 py-2 text-xs">
                <span className="font-mono text-muted-foreground">{formatTime(log.timestamp)}</span>
                <span
                  className={cn(
                    'font-semibold',
                    log.level === 'Error' && 'text-destructive',
                    log.level === 'Warn' && 'text-amber-500',
                    log.level !== 'Error' && log.level !== 'Warn' && 'text-primary'
                  )}
                >
                  {log.level.toUpperCase()}
                </span>
                <span className="break-all text-foreground/90">{log.message}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </Card>
  );
}
