import { Card, Field, Input, makeStyles, Switch, Text, tokens } from '@fluentui/react-components';
import { ArrowSyncRegular, LinkRegular, RocketRegular, TrayItemAddRegular } from '@fluentui/react-icons';
import type { ReactElement } from 'react';
import type { AppSnapshotDto, PreferenceDto } from '$lib/types/app';
import { iconSize, useShellStyles } from '$lib/features/shared/layout';

type Props = {
  snapshot: AppSnapshotDto;
  busy: boolean;
  onUpdatePreferences: (preferences: PreferenceDto) => void;
};

const useSettingsStyles = makeStyles({
  sectionCard: {
    padding: '24px',
    gap: '0'
  },
  preferenceRow: {
    display: 'grid',
    gridTemplateColumns: '32px minmax(0, 1fr) auto',
    alignItems: 'center',
    gap: '16px',
    minHeight: '64px',
    padding: '10px 0',
    borderBottom: `1px solid ${tokens.colorNeutralStroke3}`,
    ':last-child': {
      borderBottomWidth: 0
    }
  },
  icon: {
    display: 'grid',
    placeItems: 'center',
    color: tokens.colorBrandForeground1
  },
  endpointStack: {
    display: 'grid',
    gap: '16px',
    marginTop: '18px'
  }
});

export function SettingsPanel({ snapshot, busy, onUpdatePreferences }: Props) {
  const shell = useShellStyles();
  const styles = useSettingsStyles();
  const setPreference = (patch: Partial<PreferenceDto>) => {
    onUpdatePreferences({ ...snapshot.preferences, ...patch });
  };

  return (
    <section className={shell.stack} style={{ flex: 1 }}>
      <Card appearance="filled" className={styles.sectionCard}>
        <PreferenceRow
          title="关闭窗口时最小化到托盘"
          icon={<TrayItemAddRegular style={iconSize} />}
          checked={snapshot.preferences.minimizeToTrayOnClose}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ minimizeToTrayOnClose: checked })}
        />
        <PreferenceRow
          title="开机自动启动"
          icon={<RocketRegular style={iconSize} />}
          checked={snapshot.preferences.launchOnStartup}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ launchOnStartup: checked })}
        />
        <PreferenceRow
          title="流量用完后自动切换账号"
          icon={<ArrowSyncRegular style={iconSize} />}
          checked={snapshot.preferences.autoSwitchAccountOnTrafficExhausted}
          disabled={busy}
          onCheckedChange={(checked) => setPreference({ autoSwitchAccountOnTrafficExhausted: checked })}
        />
      </Card>

      <Card appearance="filled" className={styles.sectionCard}>
        <div className={shell.row}>
          <LinkRegular style={iconSize} />
          <Text size={500} weight="semibold">
            接口地址
          </Text>
        </div>
        <div className={styles.endpointStack}>
          <Field label="SRUN 认证服务地址">
            <Input readOnly value="http://rz.muc.edu.cn/srun_portal_pc.php?ac_id=1&" />
          </Field>
          <Field label="本地检测接口地址">
            <Input readOnly value="http://192.168.2.231:8800/home" />
          </Field>
        </div>
      </Card>
    </section>
  );
}

type PreferenceRowProps = {
  title: string;
  icon: ReactElement;
  checked: boolean;
  disabled: boolean;
  onCheckedChange: (checked: boolean) => void;
};

function PreferenceRow({ title, icon, checked, disabled, onCheckedChange }: PreferenceRowProps) {
  const shell = useShellStyles();
  const styles = useSettingsStyles();

  return (
    <div className={styles.preferenceRow}>
      <span className={styles.icon}>{icon}</span>
      <Text>{title}</Text>
      <Switch
        checked={checked}
        disabled={disabled}
        label={checked ? '开启' : '关闭'}
        labelPosition="before"
        onChange={(_, data) => onCheckedChange(Boolean(data.checked))}
        className={shell.muted}
      />
    </div>
  );
}
