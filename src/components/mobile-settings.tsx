import { Monitor, Moon, Sun, GitFork, Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/components/theme-provider";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getVersion } from "@tauri-apps/api/app";
import { useState, useEffect } from "react";
import { cn } from "@/lib/utils";
import { useManualUpdateCheck } from "@/components/updater-dialog";
import appIconUrl from "../../src-tauri/icons/icon.svg?url";
import type { Preferences } from "@/components/home/types";

interface MobileSettingsProps {
  preferences: Preferences | null;
  errorText: string;
  saving: boolean;
  onTogglePreference: (key: keyof Preferences) => void;
}

export function MobileSettings({
  preferences,
  errorText,
  saving,
  onTogglePreference,
}: MobileSettingsProps) {
  const { theme, setTheme } = useTheme();
  const { checkUpdate, checking, showNoUpdate, dismissNoUpdate } =
    useManualUpdateCheck();
  const [version, setVersion] = useState("");

  useEffect(() => {
    void getVersion().then(setVersion);
  }, []);

  const handleOpenGithub = async () => {
    await openUrl("https://github.com/hungryM0/MUC-student");
  };

  return (
    <div className="flex flex-col gap-6 w-full pb-8">
      {/* About Section */}
      <div className="flex flex-col items-center gap-3 pt-6 pb-4">
        <img
          src={appIconUrl}
          alt=""
          className="h-20 w-20 rounded-3xl shadow-sm border border-border/50"
        />
        <div className="text-center">
          <h2 className="text-2xl font-bold tracking-tight">MUC-student</h2>
          <p className="text-muted-foreground text-sm mt-1">版本 {version}</p>
        </div>
      </div>

      <div className="space-y-6">
        <section className="space-y-3">
          <h3 className="text-xs font-semibold tracking-wider text-muted-foreground uppercase px-2">
            版本更新
          </h3>
          <Button
            onClick={() => {
              dismissNoUpdate();
              void checkUpdate();
            }}
            className="h-11 w-full gap-2 rounded-xl"
            variant="outline"
            disabled={checking}
          >
            <Download className={cn("h-5 w-5", checking && "animate-pulse")} />
            {checking ? "检查中" : "检查更新"}
          </Button>
          {showNoUpdate && (
            <p className="px-2 text-xs text-muted-foreground">已是最新版</p>
          )}
        </section>

        {/* 外观 */}
        <section className="space-y-3">
          <h3 className="text-xs font-semibold tracking-wider text-muted-foreground uppercase px-2">
            外观主题
          </h3>
          <div className="flex gap-2">
            <Button
              variant={theme === "light" ? "default" : "outline"}
              onClick={() => setTheme("light")}
              className="flex h-11 flex-1 items-center justify-center gap-2"
            >
              <Sun className="h-5 w-5" />
              浅色
            </Button>
            <Button
              variant={theme === "dark" ? "default" : "outline"}
              onClick={() => setTheme("dark")}
              className="flex h-11 flex-1 items-center justify-center gap-2"
            >
              <Moon className="h-5 w-5" />
              深色
            </Button>
            <Button
              variant={theme === "system" ? "default" : "outline"}
              onClick={() => setTheme("system")}
              className="flex h-11 flex-1 items-center justify-center gap-2"
            >
              <Monitor className="h-5 w-5" />
              跟随系统
            </Button>
          </div>
        </section>

        {/* 行为 */}
        <section className="space-y-3">
          <h3 className="text-xs font-semibold tracking-wider text-muted-foreground uppercase px-2">
            行为配置
          </h3>
          <div className="overflow-hidden rounded-2xl border border-border/80 bg-background/60">
            <MobilePreferenceRow
              title="流量耗尽后自动切换到上一个使用的账号"
              checked={!!preferences?.autoSwitchAccountOnTrafficExhausted}
              disabled={!preferences || saving}
              onToggle={() =>
                onTogglePreference("autoSwitchAccountOnTrafficExhausted")
              }
            />
          </div>
        </section>

        {/* Links */}
        <section className="pt-2">
          <Button
            onClick={handleOpenGithub}
            className="h-11 w-full gap-2 rounded-xl"
            variant="outline"
          >
            <GitFork className="h-5 w-5" />
            GitHub 仓库
          </Button>
        </section>

        {errorText && (
          <div className="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border px-4 py-3 text-sm">
            {errorText}
          </div>
        )}
      </div>
    </div>
  );
}

function MobilePreferenceRow({
  title,
  checked,
  disabled,
  onToggle,
}: {
  title: string;
  checked: boolean;
  disabled: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      onClick={() => {
        if (!disabled) {
          onToggle();
        }
      }}
      disabled={disabled}
      className={cn(
        "flex min-h-[60px] w-full items-center justify-between gap-4 px-4 py-3 text-left transition-colors",
        disabled
          ? "cursor-not-allowed opacity-60"
          : "hover:bg-muted/40 active:bg-muted/60",
      )}
    >
      <span className="text-sm font-medium text-foreground">{title}</span>
      <span
        className={cn(
          "relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors duration-200",
          checked ? "bg-emerald-500" : "bg-slate-300 dark:bg-slate-700",
        )}
      >
        <span
          className={cn(
            "block h-5 w-5 rounded-full bg-background shadow-sm transition-transform duration-200",
            checked ? "translate-x-5.5" : "translate-x-0.5",
          )}
        />
      </span>
    </button>
  );
}
