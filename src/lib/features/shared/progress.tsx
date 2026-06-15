import { cn } from '$lib/utils';
import { progressColor } from './format';

type ProgressBarProps = {
  value: number | null | undefined;
  loading?: boolean;
  className?: string;
};

export function ProgressBar({ value, loading = false, className }: ProgressBarProps) {
  const normalized = Math.max(0, Math.min(100, value ?? 0));
  return (
    <div className={cn('h-2 overflow-hidden rounded-full bg-muted', className)}>
      {loading ? (
        <div className="h-full w-full origin-left bg-primary" style={{ animation: 'indeterminate 1.3s linear infinite' }} />
      ) : (
        <div
          className="h-full rounded-full transition-[width] duration-300"
          style={{ width: `${normalized}%`, background: progressColor(value) }}
        />
      )}
    </div>
  );
}

type TrafficRingProps = {
  value: number | null | undefined;
  className?: string;
};

export function TrafficRing({ value, className }: TrafficRingProps) {
  const normalized = Math.max(0, Math.min(100, value ?? 0));
  return (
    <div
      className={cn('traffic-ring relative grid size-20 shrink-0 place-items-center rounded-full text-sm font-semibold', className)}
      style={
        {
          '--ring-percent': `${normalized}%`,
          '--ring-color': progressColor(value)
        } as React.CSSProperties
      }
    >
      <span className="relative z-10">{value === null || value === undefined ? '--' : `${normalized.toFixed(1)}%`}</span>
    </div>
  );
}
