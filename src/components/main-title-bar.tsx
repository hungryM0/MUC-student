import { Moon, Sun, Info, Settings } from "lucide-react";
import { useTheme } from "@/components/theme-provider";
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
    onOpenAbout?.();
  };

  const handleOpenSettings = async () => {
    onOpenSettings?.();
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
