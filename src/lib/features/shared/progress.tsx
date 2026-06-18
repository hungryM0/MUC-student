import { makeStyles, ProgressBar as FluentProgressBar, tokens } from '@fluentui/react-components';
import { progressText } from './format';

type ProgressBarProps = {
  value: number | null | undefined;
  loading?: boolean;
};

export function normalizeProgress(value: number | null | undefined) {
  if (value === null || value === undefined) return 0;
  return Math.max(0, Math.min(100, value)) / 100;
}

export function ProgressBar({ value, loading = false }: ProgressBarProps) {
  return <FluentProgressBar thickness="large" value={loading ? undefined : normalizeProgress(value)} />;
}

type TrafficRingProps = {
  value: number | null | undefined;
};

const useRingStyles = makeStyles({
  root: {
    display: 'grid',
    placeItems: 'center',
    width: '128px',
    height: '128px',
    borderRadius: tokens.borderRadiusCircular,
    background: `conic-gradient(${tokens.colorBrandForeground1} var(--traffic-value), ${tokens.colorNeutralStroke2} 0)`,
    position: 'relative',
    flex: '0 0 128px',
    ':before': {
      position: 'absolute',
      inset: '12px',
      borderRadius: tokens.borderRadiusCircular,
      backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground1) 86%, transparent)',
      content: '""'
    }
  },
  value: {
    position: 'relative',
    fontSize: tokens.fontSizeBase500,
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground1
  }
});

export function TrafficRing({ value }: TrafficRingProps) {
  const styles = useRingStyles();
  const normalized = Math.max(0, Math.min(100, value ?? 0));

  return (
    <div className={styles.root} style={{ '--traffic-value': `${normalized}%` } as React.CSSProperties}>
      <span className={styles.value}>{progressText(value)}</span>
    </div>
  );
}
