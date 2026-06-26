import { useEffect, useState } from "react";
import { Monitor, Moon, RotateCw, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/components/theme-provider";
import { TitleBar } from "@/components/title-bar";
import { WindowFrame } from "@/components/window-frame";
import {
  type AppSnapshotDto,
  getAppSnapshot,
  readErrorMessage,
  updatePreferences,
} from "@/lib/muc";
import { cn } from "@/lib/utils";

type Preferences = AppSnapshotDto["preferences"];

export default function SettingsPage() {
  const { theme, setTheme } = useTheme();
  const [preferences, setPreferences] = useState<Preferences | null>(null);
  const [savingKey, setSavingKey] = useState<keyof Preferences | "">("");
  const [errorText, setErrorText] = useState("");

  useEffect(() => {
    let disposed = false;

    async function load() {
      setErrorText("");
      try {
        const snapshot = await getAppSnapshot();
        if (!disposed) {
          setPreferences(snapshot.preferences);
        }
      } catch (error) {
        if (!disposed) {
          setErrorText(readErrorMessage(error));
        }
      }
    }

    void load();
    return () => {
      disposed = true;
    };
  }, []);

  async function togglePreference(key: keyof Preferences) {
    if (!preferences || savingKey) {
      return;
    }

    const next = {
      ...preferences,
      [key]: !preferences[key],
    };
    setPreferences(next);
    setSavingKey(key);
    setErrorText("");

    try {
      const snapshot = await updatePreferences(next);
      setPreferences(snapshot.preferences);
    } catch (error) {
      setPreferences(preferences);
      setErrorText(readErrorMessage(error));
    } finally {
      setSavingKey("");
    }
  }

  return (
    <WindowFrame
      titleBar={<TitleBar title="设置" showMaximize={false} />}
      contentClassName="flex flex-1 overflow-auto bg-[radial-gradient(circle_at_top_left,rgba(14,165,233,0.06),transparent_35%),linear-gradient(135deg,rgba(16,185,129,0.03),transparent_45%)]"
    >
      <div className="w-full max-w-3xl p-6">
        <div className="space-y-6">
          <section className="space-y-3">
            <h2 className="text-lg font-semibold tracking-tight">外观</h2>
            <div className="flex flex-wrap gap-2">
              <Button
                variant={theme === "light" ? "default" : "outline"}
                size="sm"
                onClick={() => setTheme("light")}
                className="flex items-center gap-1.5"
              >
                <Sun className="h-3.5 w-3.5" />
                浅色
              </Button>
              <Button
                variant={theme === "dark" ? "default" : "outline"}
                size="sm"
                onClick={() => setTheme("dark")}
                className="flex items-center gap-1.5"
              >
                <Moon className="h-3.5 w-3.5" />
                深色
              </Button>
              <Button
                variant={theme === "system" ? "default" : "outline"}
                size="sm"
                onClick={() => setTheme("system")}
                className="flex items-center gap-1.5"
              >
                <Monitor className="h-3.5 w-3.5" />
                跟随系统
              </Button>
            </div>
          </section>

          <section className="space-y-3">
            <h2 className="text-lg font-semibold tracking-tight">行为</h2>
            <div className="divide-y divide-border/60 overflow-hidden rounded-xl border border-border/80 bg-background/50 backdrop-blur-xs">
              <PreferenceRow
                title="关闭窗口时最小化到托盘"
                checked={!!preferences?.minimizeToTrayOnClose}
                disabled={!preferences || !!savingKey}
                saving={savingKey === "minimizeToTrayOnClose"}
                onToggle={() => togglePreference("minimizeToTrayOnClose")}
              />
              <PreferenceRow
                title="开机自动启动"
                checked={!!preferences?.launchOnStartup}
                disabled={!preferences || !!savingKey}
                saving={savingKey === "launchOnStartup"}
                onToggle={() => togglePreference("launchOnStartup")}
              />
              <PreferenceRow
                title="流量用完后自动切换账号"
                checked={!!preferences?.autoSwitchAccountOnTrafficExhausted}
                disabled={!preferences || !!savingKey}
                saving={savingKey === "autoSwitchAccountOnTrafficExhausted"}
                onToggle={() =>
                  togglePreference("autoSwitchAccountOnTrafficExhausted")
                }
              />
            </div>
          </section>

          {errorText && (
            <div className="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border px-4 py-3 text-sm">
              {errorText}
            </div>
          )}
        </div>
      </div>
    </WindowFrame>
  );
}

function PreferenceRow({
  title,
  checked,
  disabled,
  saving,
  onToggle,
}: {
  title: string;
  checked: boolean;
  disabled: boolean;
  saving: boolean;
  onToggle: () => void;
}) {
  return (
    <div
      onClick={disabled ? undefined : onToggle}
      className={cn(
        "flex min-h-[52px] items-center justify-between gap-4 px-4 py-3 text-sm transition-colors select-none",
        disabled ? "cursor-not-allowed opacity-60" : "cursor-pointer hover:bg-muted/40"
      )}
    >
      <span className="font-medium text-foreground">{title}</span>
      <span className="flex items-center gap-3" onClick={(e) => e.stopPropagation()}>
        {saving && (
          <RotateCw className="text-muted-foreground h-3.5 w-3.5 animate-spin" />
        )}
        <button
          type="button"
          role="switch"
          aria-checked={checked}
          disabled={disabled}
          onClick={onToggle}
          className={cn(
            "relative inline-flex h-5.5 w-10 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 focus:outline-hidden disabled:cursor-not-allowed",
            checked ? "bg-emerald-500" : "bg-slate-300 dark:bg-slate-700",
          )}
        >
          <span
            className={cn(
              "pointer-events-none block h-4.5 w-4.5 rounded-full bg-background shadow-sm ring-0 transition-transform duration-200",
              checked ? "translate-x-5" : "translate-x-0.5",
            )}
          />
        </button>
      </span>
    </div>
  );
}
