import { Card } from '$lib/components/ui/card';
import { Input } from '$lib/components/ui/input';
import { Switch } from '$lib/components/ui/switch';
import type { AppSnapshotDto, PreferenceDto } from '$lib/types/app';
import { cn } from '$lib/utils';
import { Minimize2, Zap, Shuffle, Link2 } from 'lucide-react';

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
      {/* 偏好设置卡片 */}
      <Card className="divide-y divide-border/40 p-0 border border-border/80 shadow-sm bg-card/50 overflow-hidden">
        <PreferenceRow
          title="关闭窗口时最小化到托盘"
          icon={Minimize2}
          checked={snapshot.preferences.minimizeToTrayOnClose}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ minimizeToTrayOnClose: checked })}
        />
        <PreferenceRow
          title="开机自动启动"
          icon={Zap}
          checked={snapshot.preferences.launchOnStartup}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ launchOnStartup: checked })}
        />
        <PreferenceRow
          title="流量用完后自动切换账号"
          icon={Shuffle}
          checked={snapshot.preferences.autoSwitchAccountOnTrafficExhausted}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ autoSwitchAccountOnTrafficExhausted: checked })}
        />
      </Card>

      {/* 接口地址卡片 */}
      <Card className="grid gap-4 p-5 border border-border/80 shadow-sm bg-card/50">
        <div>
          <h3 className="text-sm font-semibold tracking-tight flex items-center gap-1.5">
            <Link2 className="size-4 text-primary" />
            接口地址
          </h3>
          <p className="text-xs text-muted-foreground mt-0.5">校园网认证服务及本地检测接口</p>
        </div>
        <div className="grid gap-3">
          <div className="space-y-1">
            <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">SRUN 认证服务地址</label>
            <Input readOnly value="http://rz.muc.edu.cn/srun_portal_pc.php?ac_id=1&" className="font-mono text-xs bg-muted/20 border-border/60" />
          </div>
          <div className="space-y-1">
            <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">本地检测接口地址</label>
            <Input readOnly value="http://192.168.2.231:8800/home" className="font-mono text-xs bg-muted/20 border-border/60" />
          </div>
        </div>
      </Card>
    </section>
  );
}

type PreferenceRowProps = {
  title: string;
  icon?: React.ElementType;
  checked: boolean;
  disabled: boolean;
  onCheckedChange: (checked: boolean) => void;
};

function PreferenceRow({ title, icon: Icon, checked, disabled, onCheckedChange }: PreferenceRowProps) {
  return (
    <label className={cn(
      "flex cursor-pointer items-center justify-between gap-4 px-5 py-4 transition-colors hover:bg-muted/30 select-none",
      "first:rounded-t-lg last:rounded-b-lg"
    )}>
      <div className="flex items-center gap-3">
        {Icon && <Icon className="size-4 text-primary shrink-0" />}
        <span className="text-sm font-medium text-foreground">{title}</span>
      </div>
      <Switch checked={checked} disabled={disabled} onCheckedChange={onCheckedChange} className="cursor-pointer" />
    </label>
  );
}
