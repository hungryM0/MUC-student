import { Monitor, Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/components/theme-provider";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { usePreferences } from "@/hooks/use-preferences";
import { cn } from "@/lib/utils";

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
}

export function SettingsDialog({ open, onClose }: SettingsDialogProps) {
  const { theme, setTheme } = useTheme();
  const { preferences, errorText, togglePreference } = usePreferences(open);

  return (
    <Dialog open={open} onOpenChange={(val) => !val && onClose()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>设置</DialogTitle>
        </DialogHeader>

        <div className="space-y-5 py-2">
          {/* 外观 */}
          <section className="space-y-2.5">
            <h3 className="text-xs font-semibold tracking-wider text-muted-foreground uppercase">
              外观主题
            </h3>
            <div className="flex gap-2">
              <Button
                variant={theme === "light" ? "default" : "outline"}
                size="sm"
                onClick={() => setTheme("light")}
                className="flex-1 flex items-center justify-center gap-1.5 h-8.5"
              >
                <Sun className="h-3.5 w-3.5" />
                浅色
              </Button>
              <Button
                variant={theme === "dark" ? "default" : "outline"}
                size="sm"
                onClick={() => setTheme("dark")}
                className="flex-1 flex items-center justify-center gap-1.5 h-8.5"
              >
                <Moon className="h-3.5 w-3.5" />
                深色
              </Button>
              <Button
                variant={theme === "system" ? "default" : "outline"}
                size="sm"
                onClick={() => setTheme("system")}
                className="flex-1 flex items-center justify-center gap-1.5 h-8.5 text-xs"
              >
                <Monitor className="h-3.5 w-3.5" />
                跟随系统
              </Button>
            </div>
          </section>

          {/* 行为 */}
          <section className="space-y-2.5">
            <h3 className="text-xs font-semibold tracking-wider text-muted-foreground uppercase">
              行为配置
            </h3>
            <div className="divide-y divide-border/60 overflow-hidden rounded-xl border border-border/80 bg-background/50">
              <PreferenceRow
                title="关闭窗口时最小化到托盘"
                checked={!!preferences?.minimizeToTrayOnClose}
                disabled={!preferences}
                onToggle={() => togglePreference("minimizeToTrayOnClose")}
              />
              <PreferenceRow
                title="开机自动启动"
                checked={!!preferences?.launchOnStartup}
                disabled={!preferences}
                onToggle={() => togglePreference("launchOnStartup")}
              />
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

          {errorText && (
            <div className="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border px-3 py-2 text-xs">
              {errorText}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
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
        "flex min-h-[48px] items-center justify-between gap-4 px-4 py-2.5 text-xs transition-colors select-none",
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
            "relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 focus:outline-hidden disabled:cursor-not-allowed",
            checked ? "bg-emerald-500" : "bg-slate-300 dark:bg-slate-700",
          )}
        >
          <span
            className={cn(
              "pointer-events-none block h-4 w-4 rounded-full bg-background shadow-sm ring-0 transition-transform duration-200",
              checked ? "translate-x-4.5" : "translate-x-0.5",
            )}
          />
        </button>
      </span>
    </div>
  );
}
