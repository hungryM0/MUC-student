import { Button } from "@/components/ui/button";
import { useTheme } from "@/components/theme-provider";
import { TitleBar } from "@/components/title-bar";
import { WindowFrame } from "@/components/window-frame";
import { Moon, Sun, Monitor } from "lucide-react";

export default function SettingsPage() {
  const { theme, setTheme } = useTheme();

  return (
    <WindowFrame
      titleBar={<TitleBar title="设置" showMaximize={false} />}
      contentClassName="flex flex-1 overflow-auto"
    >
      <div className="w-full max-w-3xl p-4">
        <div className="space-y-4">
          <h2 className="mb-1 text-lg font-semibold">外观</h2>

          <div className="space-y-0">
            <div className="flex items-center justify-between py-2.5">
              <label className="text-sm font-medium">主题</label>
              <div className="flex gap-2">
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
            </div>
          </div>
        </div>
      </div>
    </WindowFrame>
  );
}
