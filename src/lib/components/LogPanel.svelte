<script lang="ts">
  import type { LogItemDto } from '$lib/types/app';

  type Props = {
    logs: LogItemDto[];
  };

  let { logs }: Props = $props();

  const timeText = (value: string) => {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return '--:--:--';
    return date.toLocaleTimeString('zh-CN', { hour12: false });
  };

  const levelClass = (level: string) => {
    const normalized = level.toLowerCase();
    if (normalized.includes('success')) return 'success';
    if (normalized.includes('warn')) return 'warn';
    if (normalized.includes('error')) return 'error';
    return '';
  };
</script>

<section class="card log-panel">
  <div class="card-header">
    <div>
      <h2 class="card-title">登录日志</h2>
      <div class="card-subtitle">只展示后端事件，不在前端拼业务结论</div>
    </div>
    <span class="pill">{logs.length} 条</span>
  </div>
  <div class="card-body">
    {#if logs.length === 0}
      <div class="inline-note">暂无日志</div>
    {:else}
      <div class="log-list">
        {#each logs.slice().reverse() as log (`${log.timestamp}-${log.level}-${log.message}`)}
          <div class="log-entry">
            <span class="log-time mono">{timeText(log.timestamp)}</span>
            <span class={`log-level ${levelClass(log.level)}`}>{log.level}</span>
            <span>{log.message}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</section>
