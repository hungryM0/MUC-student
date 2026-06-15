import { Card } from '$lib/components/ui/card';
import { Input } from '$lib/components/ui/input';
import { Switch } from '$lib/components/ui/switch';
import type { AppSnapshotDto, PreferenceDto } from '$lib/types/app';

type Props = {
  snapshot: AppSnapshotDto;
  busy: boolean;
  onUpdatePreferences: (preferences: PreferenceDto) => void;
};

export function SettingsPanel({ snapshot, busy, onUpdatePreferences }: Props) {
  const setPreference = (patch: Partial<PreferenceDto>) => {
    onUpdatePreferences({ ...snapshot.preferences, ...patch });
  };

  return (
    <section className="panel-in flex min-h-0 flex-1 flex-col gap-4">
      <h2 className="text-xl font-semibold">设置</h2>
      <Card className="divide-y divide-border p-5">
        <PreferenceRow
          title="关闭窗口时最小化到托盘"
          checked={snapshot.preferences.minimizeToTrayOnClose}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ minimizeToTrayOnClose: checked })}
        />
        <PreferenceRow
          title="开机自动启动"
          checked={snapshot.preferences.launchOnStartup}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ launchOnStartup: checked })}
        />
        <PreferenceRow
          title="流量用完后自动切换账号"
          checked={snapshot.preferences.autoSwitchAccountOnTrafficExhausted}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ autoSwitchAccountOnTrafficExhausted: checked })}
        />
      </Card>

      <Card className="grid gap-4 p-5">
        <h3 className="text-base font-semibold">接口地址</h3>
        <div className="grid gap-3">
          <Input readOnly value="http://rz.muc.edu.cn/srun_portal_pc.php?ac_id=1&" className="font-mono text-xs" />
          <Input readOnly value="http://192.168.2.231:8800/home" className="font-mono text-xs" />
        </div>
      </Card>
    </section>
  );
}

type PreferenceRowProps = {
  title: string;
  checked: boolean;
  disabled: boolean;
  onCheckedChange: (checked: boolean) => void;
};

function PreferenceRow({ title, checked, disabled, onCheckedChange }: PreferenceRowProps) {
  return (
    <label className="flex cursor-pointer items-center justify-between gap-4 py-4 first:pt-0 last:pb-0">
      <span className="text-sm font-medium">{title}</span>
      <Switch checked={checked} disabled={disabled} onCheckedChange={onCheckedChange} />
    </label>
  );
}
