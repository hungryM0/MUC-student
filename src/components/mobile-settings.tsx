import { Monitor, Moon, Sun, GitFork } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/components/theme-provider";
import { usePreferences } from "@/hooks/use-preferences";
import { cn } from "@/lib/utils";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getVersion } from "@tauri-apps/api/app";
import { useState, useEffect } from "react";
import appIconUrl from "../../src-tauri/icons/icon.svg?url";

export function MobileSettings() {
  const { theme, setTheme } = useTheme();
  const { preferences, errorText, togglePreference } = usePreferences(true);
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
          <div className="divide-y divide-border/60 overflow-hidden rounded-xl border border-border/80 bg-background/50">
            <PreferenceRow
              title="流量用完后自动切换账号"
              checked={!!preferences?.autoSwitchAccountOnTrafficExhausted}
              disabled={!preferences}
              onToggle={() =>
                togglePreference("autoSwitchAccountOnTrafficExhausted")
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

function PreferenceRow({
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
    <div
      onClick={disabled ? undefined : onToggle}
      className={cn(
        "flex min-h-[56px] items-center justify-between gap-4 px-4 py-3 text-sm transition-colors select-none",
        disabled
          ? "cursor-not-allowed opacity-60"
          : "cursor-pointer hover:bg-muted/40",
      )}
    >
      <span className="font-medium text-foreground">{title}</span>
      <span
        className="flex items-center gap-2.5"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          type="button"
          role="switch"
          aria-checked={checked}
          disabled={disabled}
          onClick={onToggle}
          className={cn(
            "relative inline-flex h-6 w-10 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 focus:outline-hidden disabled:cursor-not-allowed",
            checked ? "bg-emerald-500" : "bg-slate-300 dark:bg-slate-700",
          )}
        >
          <span
            className={cn(
              "pointer-events-none block h-5 w-5 rounded-full bg-background shadow-sm ring-0 transition-transform duration-200",
              checked ? "translate-x-5" : "translate-x-0.5",
            )}
          />
        </button>
      </span>
    </div>
  );
}
