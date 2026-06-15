<script lang="ts">
  import type { AccountDto } from '$lib/types/app';

  type Props = {
    account: AccountDto;
    selected?: boolean;
    actionEnabled?: boolean;
    compact?: boolean;
    onEdit?: (accountId: string) => void;
    onDelete?: (accountId: string) => void;
    onLogout?: (accountId: string) => void;
  };

  let {
    account,
    selected = false,
    actionEnabled = true,
    compact = false,
    onEdit,
    onDelete,
    onLogout
  }: Props = $props();

  const displayPercent = (value: number | null | undefined) =>
    typeof value === 'number' ? Math.max(0, Math.min(100, value)) : null;

  const ringLabel = (value: number | null | undefined) => {
    const percent = displayPercent(value);
    return percent === null ? '--' : `${percent.toFixed(1)}%`;
  };

  const ringStyle = (value: number | null | undefined) => {
    const percent = displayPercent(value) ?? 0;
    return `--ring-percent: ${percent}%`;
  };

  const isExhausted = (value: number | null | undefined) => typeof value === 'number' && value >= 100;
</script>

<article class:selected class="list-item account-card" class:compact>
  <div class="ring" class:small={compact} style={ringStyle(account.snapshot?.progressPercent)}>
    {ringLabel(account.snapshot?.progressPercent)}
  </div>

  <div class="stack tight">
    <div class="split">
      <div>
        <div class="row">
          <strong>{account.remarkName || account.username}</strong>
          {#if account.isCurrentOnline}
            <span class="pill success">本机在线</span>
          {/if}
          {#if selected}
            <span class="pill">登录目标</span>
          {/if}
          {#if isExhausted(account.snapshot?.progressPercent)}
            <span class="pill danger">已用尽</span>
          {/if}
        </div>
        <div class="inline-note mono">账号：{account.username}</div>
      </div>
      {#if !compact}
        <div class="row">
          <button class="action-button secondary" type="button" onclick={() => onEdit?.(account.id)}>编辑</button>
          <button class="action-button ghost" type="button" onclick={() => onDelete?.(account.id)}>删除</button>
          {#if account.canLogoutLocalDevice}
            <button class="action-button warn" disabled={!actionEnabled} type="button" onclick={() => onLogout?.(account.id)}>
              下线本机
            </button>
          {/if}
        </div>
      {/if}
    </div>

    <div class="grid-3">
      <div class="metric">
        <span class="metric-label">在线设备</span>
        <span class="metric-value">{account.snapshot?.onlineDeviceCountText || '-'}</span>
      </div>
      <div class="metric">
        <span class="metric-label">已用流量</span>
        <span class="metric-value">{account.snapshot?.usedTrafficText || '-'}</span>
      </div>
      <div class="metric">
        <span class="metric-label">账户总流量</span>
        <span class="metric-value">{account.snapshot?.productBalanceText || '-'}</span>
      </div>
    </div>

    {#if account.snapshot?.includedPackageText || account.snapshot?.detailText}
      <div class="inline-note">
        {#if account.snapshot?.includedPackageText}<span class="pill success">{account.snapshot.includedPackageText}</span>{/if}
        {#if account.snapshot?.detailText}<span>{account.snapshot.detailText}</span>{/if}
      </div>
    {/if}
  </div>
</article>
