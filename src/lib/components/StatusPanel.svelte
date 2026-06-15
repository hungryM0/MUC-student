<script lang="ts">
  import AccountCard from '$lib/components/AccountCard.svelte';
  import QuotaSummaryCard from '$lib/components/QuotaSummaryCard.svelte';
  import type { AccountDto, AppSnapshotDto, UiState } from '$lib/types/app';

  type Props = {
    snapshot: AppSnapshotDto;
    sortMode: UiState['sortMode'];
    busy?: boolean;
    onSortMode: (mode: UiState['sortMode']) => void;
    onAdd: () => void;
    onEdit: (accountId: string) => void;
    onDelete: (accountId: string) => void;
    onLogout: (accountId: string) => void;
    onRefresh: () => void;
  };

  let {
    snapshot,
    sortMode,
    busy = false,
    onSortMode,
    onAdd,
    onEdit,
    onDelete,
    onLogout,
    onRefresh
  }: Props = $props();

  const sortedAccounts = () => {
    const accounts = [...snapshot.accounts];
    if (sortMode === 'remainingDesc') {
      accounts.sort((a, b) => remainingSortKey(b) - remainingSortKey(a));
    }
    if (sortMode === 'nameAsc') {
      accounts.sort((a, b) => `${a.remarkName}${a.username}`.localeCompare(`${b.remarkName}${b.username}`, 'zh-CN'));
    }
    return orderByCurrentOnline(accounts);
  };

  function orderByCurrentOnline(accounts: AccountDto[]) {
    const current = accounts.filter((account) => account.isCurrentOnline);
    const exhausted = accounts.filter((account) => !account.isCurrentOnline && (account.snapshot?.progressPercent ?? -1) >= 100);
    const active = accounts.filter((account) => !account.isCurrentOnline && (account.snapshot?.progressPercent ?? -1) < 100);
    return [...current, ...exhausted, ...active];
  }

  function remainingSortKey(account: AccountDto) {
    const percent = account.snapshot?.progressPercent;
    if (typeof percent !== 'number') return -1;
    return 100 - percent;
  }
</script>

<div class="page-grid status fade-in">
  <QuotaSummaryCard title="号池总配额" quota={snapshot.poolQuota} loading={snapshot.refreshState.running} />

  <section class="card">
    <div class="card-header">
      <div>
        <h2 class="card-title">状态</h2>
        <div class="card-subtitle">账号快照、在线设备、本机下线入口</div>
      </div>
      <div class="row">
        <select class="select" style="width: 180px" value={sortMode} onchange={(event) => onSortMode(event.currentTarget.value as UiState['sortMode'])}>
          <option value="default">默认排序</option>
          <option value="remainingDesc">剩余量从高到低</option>
          <option value="nameAsc">姓名 A-Z</option>
        </select>
        <button class="action-button secondary" disabled={busy || snapshot.refreshState.running} type="button" onclick={onRefresh}>
          {snapshot.refreshState.running ? '刷新中...' : '刷新状态'}
        </button>
        <button class="action-button" type="button" onclick={onAdd}>添加账号</button>
      </div>
    </div>

    <div class="card-body">
      {#if snapshot.accounts.length === 0}
        <div class="inline-note">还没有账号，先点右上角添加账号。</div>
      {:else}
        <div class="list">
          {#each sortedAccounts() as account (account.id)}
            <AccountCard
              {account}
              selected={account.id === snapshot.selectedAccountId}
              actionEnabled={!busy && !snapshot.refreshState.running && !snapshot.loginState.running}
              onEdit={onEdit}
              onDelete={onDelete}
              onLogout={onLogout}
            />
          {/each}
        </div>
      {/if}
    </div>
  </section>
</div>
