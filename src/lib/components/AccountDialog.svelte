<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { AccountDto, AccountInput, AccountUpdateInput } from '$lib/types/app';

  type SubmitDetail = AccountInput | AccountUpdateInput;
  type Props = {
    mode: 'create' | 'edit';
    account?: AccountDto | null;
    busy?: boolean;
  };

  let { mode, account = null, busy = false }: Props = $props();
  const dispatch = createEventDispatcher<{ close: void; submit: SubmitDetail }>();

  let remarkName = $state('');
  let username = $state('');
  let password = $state('');
  let localError = $state('');

  $effect(() => {
    remarkName = account?.remarkName ?? '';
    username = account?.username ?? '';
    password = '';
  });

  function submit() {
    localError = '';
    const remark = remarkName.trim();
    const user = username.trim();
    if (!remark || !user) {
      localError = '请填写备注名和账号';
      return;
    }
    if (mode === 'create' && !password) {
      localError = '请填写密码';
      return;
    }
    if (mode === 'edit' && account) {
      dispatch('submit', {
        accountId: account.id,
        remarkName: remark,
        username: user,
        password: password ? password : null
      });
      return;
    }
    dispatch('submit', { remarkName: remark, username: user, password });
  }
</script>

<div class="overlay" role="presentation" onclick={(event) => event.target === event.currentTarget && dispatch('close')}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="account-dialog-title">
    <div class="dialog-header">
      <div>
        <h2 class="card-title" id="account-dialog-title">{mode === 'create' ? '添加账号' : '编辑账号'}</h2>
      </div>
      <button class="action-button ghost" type="button" onclick={() => dispatch('close')}>关闭</button>
    </div>
    <div class="dialog-body">
      <form class="stack" onsubmit={(event) => { event.preventDefault(); submit(); }}>
        <div class="form-grid">
          <label class="form-field">
            <span class="field-label">备注名</span>
            <input class="input" bind:value={remarkName} autocomplete="off" />
          </label>
          <label class="form-field">
            <span class="field-label">账号</span>
            <input class="input" bind:value={username} autocomplete="username" />
          </label>
          <label class="form-field full">
            <span class="field-label">密码</span>
            <input
              class="input"
              bind:value={password}
              type="password"
              autocomplete={mode === 'create' ? 'new-password' : 'current-password'}
              placeholder={mode === 'edit' ? '不填则保留原密码' : ''}
            />
          </label>
        </div>
        {#if localError}
          <div class="error-banner"><strong>{localError}</strong></div>
        {/if}
        <div class="dialog-actions">
          <button class="action-button secondary" type="button" onclick={() => dispatch('close')}>取消</button>
          <button class="action-button" disabled={busy} type="submit">保存</button>
        </div>
      </form>
    </div>
  </div>
</div>
