<script lang="ts">
  import type { PoolQuotaDto } from '$lib/types/app';

  type Props = {
    title: string;
    quota: PoolQuotaDto;
    loading?: boolean;
  };

  let { title, quota, loading = false }: Props = $props();

  const percentText = (value: number | null) => (value === null ? '--' : `${value.toFixed(1)}%`);
  const percentWidth = (value: number | null) => `${Math.max(0, Math.min(100, value ?? 0))}%`;
</script>

<section class="card">
  <div class="card-header">
    <div>
      <h2 class="card-title">{title}</h2>
      <div class="card-subtitle">{loading ? '刷新中...' : '最近一次缓存状态'}</div>
    </div>
    <span class="pill" class:warn={loading}>{percentText(quota.progressPercent)}</span>
  </div>
  <div class="card-body stack">
    <div class="progress-bar" aria-label="配额进度">
      <span style={`width: ${percentWidth(quota.progressPercent)}`}></span>
    </div>
    <div class="grid-2">
      <div class="metric">
        <span class="metric-label">已用流量</span>
        <span class="metric-value">{quota.usedTrafficText || '-'}</span>
      </div>
      <div class="metric">
        <span class="metric-label">总流量</span>
        <span class="metric-value">{quota.productBalanceText || '-'}</span>
      </div>
    </div>
    {#if quota.includedPackageText}
      <span class="pill success">{quota.includedPackageText}</span>
    {/if}
  </div>
</section>
