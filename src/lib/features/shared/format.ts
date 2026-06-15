export function formatDateTime(value: string | null | undefined) {
  if (!value) return '-';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString('zh-CN', { hour12: false });
}

export function formatTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

export function progressColor(percent: number | null | undefined) {
  if (percent === null || percent === undefined) return 'hsl(var(--primary))';
  if (percent <= 50) return 'hsl(var(--primary))';
  if (percent <= 75) return '#d99212';
  if (percent <= 90) return '#e85d04';
  return 'hsl(var(--destructive))';
}

export function progressText(percent: number | null | undefined) {
  if (percent === null || percent === undefined) return '--';
  return `${Math.max(0, Math.min(100, percent)).toFixed(1)}%`;
}
