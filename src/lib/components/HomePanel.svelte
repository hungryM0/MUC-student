<script lang="ts">
  import AccountCard from '$lib/components/AccountCard.svelte';
  import LogPanel from '$lib/components/LogPanel.svelte';
  import QuotaSummaryCard from '$lib/components/QuotaSummaryCard.svelte';
  import type { AppSnapshotDto } from '$lib/types/app';

  type Props = {
    snapshot: AppSnapshotDto;
    busy?: boolean;
    onSelectAccount: (accountId: string) => void;
    onLogin: () => void;
  };

  let { snapshot, busy = false, onSelectAccount, onLogin }: Props = $props();

  const selectedAccount = () => snapshot.accounts.find((account) => account.id === snapshot.selectedAccountId) ?? null;
  const loginTimeText = () => formatDate(snapshot.loginState.lastLoginTime);

  function formatDate(value: string | null) {
    if (!value) return '-';
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleString('zh-CN', { hour12: false });
  }
</script>

<div class="page-grid home fade-in">
  <div class="stack">
    <section class="card">
      <div class="card-header">
        <div>
          <h2 class="card-title">主页</h2>
          <div class="card-subtitle">选择账号后执行校园网 HTTP 认证</div>
        </div>
        <span class="pill" class:warn={snapshot.loginState.running}>{snapshot.loginState.resultText}</span>
      </div>
      <div class="card-body stack">
        <div class="grid-2">
          <div class="metric">
            <span class="metric-label">当前内网 IPv4</span>
            <span class="metric-value mono">{snapshot.network.ip}</span>
          </div>
          <div class="metric">
            <span class="metric-label">最近一次登录时间</span>
            <span class="metric-value mono">{loginTimeText()}</span>
          </div>
        </div>

        <div class="form-field">
          <label class="field-label" for="account-select">登录账号</label>
          <select
            class="select"
            id="account-select"
            disabled={busy || snapshot.loginState.running || snapshot.accounts.length === 0}
            value={snapshot.selectedAccountId}
            onchange={(event) => onSelectAccount(event.currentTarget.value)}
          >
            {#if snapshot.accounts.length === 0}
              <option value="">请先在状态页添加账号</option>
            {:else}
              {#each snapshot.accounts as account}
                <option value={account.id}>{account.remarkName || account.username}（{account.username}）</option>
              {/each}
            {/if}
          </select>
        </div>

        <div class="row">
          <button
            class="action-button"
            type="button"
            disabled={busy || snapshot.loginState.running || snapshot.accounts.length === 0}
            onclick={onLogin}
          >
            {snapshot.loginState.running ? '登录中...' : snapshot.accounts.length === 0 ? '请先添加账号' : '开始登录'}
          </button>
          <span class="inline-note">{snapshot.loginState.message}</span>
        </div>
      </div>
    </section>

    {#if selectedAccount()}
      <AccountCard account={selectedAccount()!} selected compact actionEnabled={!busy} />
    {/if}

    <QuotaSummaryCard title="账号配额" quota={snapshot.poolQuota} loading={snapshot.refreshState.running} />
  </div>

  <LogPanel logs={snapshot.logs} />
</div>
