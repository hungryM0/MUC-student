<script lang="ts">
  import { onMount } from 'svelte';
  import AccountDialog from '$lib/components/AccountDialog.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import HomePanel from '$lib/components/HomePanel.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import StatusPanel from '$lib/components/StatusPanel.svelte';
  import {
    appSnapshot,
    bootstrapApp,
    clearError,
    closeDialog,
    createAccount,
    deleteAccount,
    dialogState,
    findAccount,
    loginSelectedAccount,
    logoutLocalDevice,
    openCreateAccountDialog,
    openDeleteConfirm,
    openEditAccountDialog,
    openLogoutConfirm,
    refreshDashboard,
    selectAccount,
    setActivePage,
    setSortMode,
    uiState,
    updateAccount,
    updatePreferences
  } from '$lib/stores/app';
  import type { AccountInput, AccountUpdateInput, PreferenceInput } from '$lib/types/app';

  onMount(() => {
    bootstrapApp().catch(() => undefined);
  });

  const busy = () => Boolean($uiState.loadingMessage || $appSnapshot?.loginState.running || $appSnapshot?.refreshState.running);

  async function submitAccount(event: CustomEvent<AccountInput | AccountUpdateInput>) {
    try {
      if ('accountId' in event.detail) {
        await updateAccount(event.detail);
      } else {
        await createAccount(event.detail);
      }
      closeDialog();
    } catch {
      // 错误已写入 uiState。
    }
  }

  async function confirmDelete(accountId: string) {
    try {
      await deleteAccount(accountId);
      closeDialog();
    } catch {
      // 错误已写入 uiState。
    }
  }

  async function confirmLogout() {
    try {
      await logoutLocalDevice();
      closeDialog();
    } catch {
      // 错误已写入 uiState。
    }
  }

  async function savePreferences(preferences: PreferenceInput) {
    try {
      await updatePreferences(preferences);
    } catch {
      // 错误已写入 uiState。
    }
  }
</script>

{#if !$appSnapshot}
  <main class="app-shell">
    <section class="shell-frame">
      <div class="shell-content" style="justify-content: center; align-items: center; text-align: center">
        <h1 class="brand-title">MUC-student</h1>
        <span class="pill warn">{$uiState.loadingMessage || '启动中...'}</span>
        {#if $uiState.error}
          <div class="error-banner" style="max-width: 560px; text-align: left">
            <div>
              <strong>{$uiState.error.message}</strong>
              <span>{$uiState.error.detail}</span>
            </div>
          </div>
        {/if}
      </div>
    </section>
  </main>
{:else}
  <main class="app-shell">
    <section class="shell-frame">
      <div class="shell-content">
        <header class="topbar">
          <div class="brand-block">
            <h1 class="brand-title">MUC-student</h1>
            <div class="brand-subtitle">校园网多账号登录与流量状态</div>
          </div>
          <div class="topbar-meta">
            <span class="status-chip">
              <span class:warn={!$appSnapshot.network.isOnline} class="status-dot"></span>
              <strong>{$appSnapshot.network.statusText}</strong>
            </span>
            <span class="status-chip mono">IPv4 <strong>{$appSnapshot.network.ip}</strong></span>
            {#if $uiState.loadingMessage}
              <span class="status-chip"><span class="status-dot warn"></span>{$uiState.loadingMessage}</span>
            {/if}
          </div>
        </header>

        <nav class="nav-tabs" aria-label="主导航">
          <button class:active={$uiState.activePage === 'home'} class="tab-button" type="button" onclick={() => setActivePage('home')}>主页</button>
          <button class:active={$uiState.activePage === 'status'} class="tab-button" type="button" onclick={() => setActivePage('status')}>状态</button>
          <button class:active={$uiState.activePage === 'settings'} class="tab-button" type="button" onclick={() => setActivePage('settings')}>设置</button>
        </nav>

        {#if $uiState.error}
          <div class="error-banner">
            <div style="flex: 1">
              <strong>{$uiState.error.message}</strong>
              <span>{$uiState.error.detail}</span>
            </div>
            <button class="action-button ghost" type="button" onclick={clearError}>关闭</button>
          </div>
        {/if}

        {#if $uiState.activePage === 'home'}
          <HomePanel
            snapshot={$appSnapshot}
            busy={busy()}
            onSelectAccount={(accountId) => selectAccount(accountId).catch(() => undefined)}
            onLogin={() => loginSelectedAccount().catch(() => undefined)}
          />
        {:else if $uiState.activePage === 'status'}
          <StatusPanel
            snapshot={$appSnapshot}
            sortMode={$uiState.sortMode}
            busy={busy()}
            onSortMode={setSortMode}
            onAdd={openCreateAccountDialog}
            onEdit={openEditAccountDialog}
            onDelete={openDeleteConfirm}
            onLogout={openLogoutConfirm}
            onRefresh={() => refreshDashboard().catch(() => undefined)}
          />
        {:else}
          <SettingsPanel snapshot={$appSnapshot} busy={busy()} onUpdatePreferences={savePreferences} />
        {/if}
      </div>
    </section>
  </main>

  {#if $dialogState.type === 'account'}
    <AccountDialog
      mode={$dialogState.mode}
      account={$dialogState.mode === 'edit' ? findAccount($appSnapshot, $dialogState.accountId) : null}
      busy={busy()}
      on:close={closeDialog}
      on:submit={submitAccount}
    />
  {:else if $dialogState.type === 'confirmDelete'}
    <ConfirmDialog
      title="删除账号"
      message="确定删除这个账号吗？删掉后需要重新添加才能再用。"
      confirmText="删除"
      danger
      busy={busy()}
      on:close={closeDialog}
      on:confirm={() => confirmDelete($dialogState.accountId)}
    />
  {:else if $dialogState.type === 'confirmLogout'}
    <ConfirmDialog
      title="下线本机设备"
      message="下线后本机将会断网，需要重新登录认证。该账号上其他在线设备不受影响。"
      confirmText="确认下线"
      busy={busy()}
      on:close={closeDialog}
      on:confirm={confirmLogout}
    />
  {/if}
{/if}
