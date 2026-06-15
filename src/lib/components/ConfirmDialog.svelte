<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  type Props = {
    title: string;
    message: string;
    confirmText: string;
    danger?: boolean;
    busy?: boolean;
  };

  let { title, message, confirmText, danger = false, busy = false }: Props = $props();
  const dispatch = createEventDispatcher<{ close: void; confirm: void }>();
</script>

<div class="overlay" role="presentation" onclick={(event) => event.target === event.currentTarget && dispatch('close')}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="confirm-dialog-title">
    <div class="dialog-header">
      <h2 class="card-title" id="confirm-dialog-title">{title}</h2>
      <button class="action-button ghost" type="button" onclick={() => dispatch('close')}>关闭</button>
    </div>
    <div class="dialog-body">
      <p class="inline-note" style="font-size: 1rem; color: var(--text); line-height: 1.6">{message}</p>
      <div class="dialog-actions">
        <button class="action-button secondary" type="button" onclick={() => dispatch('close')}>取消</button>
        <button class:danger class="action-button" disabled={busy} type="button" onclick={() => dispatch('confirm')}>
          {confirmText}
        </button>
      </div>
    </div>
  </div>
</div>
