<script lang="ts">
  import type { AppSnapshotDto, PreferenceDto } from '$lib/types/app';

  type Props = {
    snapshot: AppSnapshotDto;
    busy?: boolean;
    onUpdatePreferences: (preferences: PreferenceDto) => void;
  };

  let { snapshot, busy = false, onUpdatePreferences }: Props = $props();

  const setPreference = (patch: Partial<PreferenceDto>) => {
    onUpdatePreferences({ ...snapshot.preferences, ...patch });
  };
</script>

<div class="page-grid settings fade-in">
  <section class="card">
    <div class="card-header">
      <div>
        <h2 class="card-title">设置</h2>
        <div class="card-subtitle">只保留功能等价需要的三个偏好项</div>
      </div>
    </div>
    <div class="card-body stack">
      <label class="list-item split">
        <div>
          <strong>关闭窗口时最小化到托盘</strong>
          <div class="inline-note">开启后，关闭主窗口只隐藏到系统托盘。</div>
        </div>
        <input
          style="width: auto"
          type="checkbox"
          disabled={busy}
          checked={snapshot.preferences.minimizeToTrayOnClose}
          onchange={(event) => setPreference({ minimizeToTrayOnClose: event.currentTarget.checked })}
        />
      </label>

      <label class="list-item split">
        <div>
          <strong>开机自动启动</strong>
          <div class="inline-note">登录 Windows 时自动启动程序。</div>
        </div>
        <input
          style="width: auto"
          type="checkbox"
          disabled={busy}
          checked={snapshot.preferences.launchOnStartup}
          onchange={(event) => setPreference({ launchOnStartup: event.currentTarget.checked })}
        />
      </label>

      <label class="list-item split">
        <div>
          <strong>流量用完后自动切换账号</strong>
          <div class="inline-note">按最近使用优先且未用尽流量的现有规则切换。</div>
        </div>
        <input
          style="width: auto"
          type="checkbox"
          disabled={busy}
          checked={snapshot.preferences.autoSwitchAccountOnTrafficExhausted}
          onchange={(event) => setPreference({ autoSwitchAccountOnTrafficExhausted: event.currentTarget.checked })}
        />
      </label>
    </div>
  </section>

  <section class="card">
    <div class="card-header">
      <div>
        <h2 class="card-title">接口地址</h2>
        <div class="card-subtitle">认证网关和自助面板严格分开</div>
      </div>
    </div>
    <div class="card-body grid-2">
      <div class="metric">
        <span class="metric-label">认证 URL</span>
        <span class="metric-value mono">http://rz.muc.edu.cn/srun_portal_pc.php?ac_id=1&amp;</span>
      </div>
      <div class="metric">
        <span class="metric-label">流量查询 URL</span>
        <span class="metric-value mono">http://192.168.2.231:8800/home</span>
      </div>
    </div>
  </section>
</div>
