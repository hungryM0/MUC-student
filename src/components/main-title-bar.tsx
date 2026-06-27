import { Moon, Sun, Info, Settings } from "lucide-react";
import { useTheme } from "@/components/theme-provider";
import { createWindow } from "@/lib/window";
import { TitleBar } from "@/components/title-bar";

interface MainTitleBarProps {
  onOpenSettings?: () => void;
  onOpenAbout?: () => void;
}

export function MainTitleBar({
  onOpenSettings,
  onOpenAbout,
}: MainTitleBarProps) {
  const { theme, setTheme } = useTheme();

  const handleToggleTheme = () => {
    setTheme(theme === "dark" ? "light" : "dark");
  };

  const handleOpenAbout = async () => {
    if (onOpenAbout) {
      onOpenAbout();
    } else {
      await createWindow("about", {
        title: "关于",
        url: "/about",
        width: 500,
        height: 400,
        resizable: false,
        maximizable: false,
        minimizable: false,
        decorations: false,
        transparent: true,
        shadow: true,
        alwaysOnTop: true,
        parent: "main",
      });
    }
  };

  const handleOpenSettings = async () => {
    if (onOpenSettings) {
      onOpenSettings();
    } else {
      await createWindow("settings", {
        title: "设置",
        url: "/settings",
        width: 600,
        height: 500,
        resizable: true,
        maximizable: true,
        minimizable: false,
        decorations: false,
        transparent: true,
        shadow: true,
        parent: "main",
      });
    }
  };

  return (
    <TitleBar
      title="MUC-student"
      rightActions={
        <>
          <button
            onClick={handleOpenSettings}
            className="title-bar-btn mr-1"
            aria-label="设置"
            tabIndex={-1}
          >
            <Settings className="h-4 w-4" />
          </button>

          <button
            onClick={handleOpenAbout}
            className="title-bar-btn mr-1"
            aria-label="关于"
            tabIndex={-1}
          >
            <Info className="h-4 w-4" />
          </button>

          <button
            onClick={handleToggleTheme}
            className="title-bar-btn mr-0.5"
            aria-label="切换主题"
            tabIndex={-1}
          >
            {theme === "dark" ? (
              <Sun className="h-4 w-4" />
            ) : (
              <Moon className="h-4 w-4" />
            )}
          </button>
        </>
      }
    />
  );
}
