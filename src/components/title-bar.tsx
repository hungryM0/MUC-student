import { useEffect, useState, ReactNode } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Minus, Maximize2, Minimize2, X } from "lucide-react";
import { cn, isAndroid } from "@/lib/utils";

interface TitleBarProps {
  title?: string;
  showMinimize?: boolean;
  showMaximize?: boolean;
  showClose?: boolean;
  leftActions?: ReactNode;
  rightActions?: ReactNode;
  onDoubleClick?: () => void;
}

export function TitleBar({
  title,
  showMinimize = true,
  showMaximize = true,
  showClose = true,
  leftActions,
  rightActions,
  onDoubleClick,
}: TitleBarProps) {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    if (!showMaximize) return;

    const appWindow = getCurrentWebviewWindow();

    // Initialize maximized state
    appWindow.isMaximized().then(setIsMaximized);

    // Listen for window resize events
    const unlisten = appWindow.onResized(async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [showMaximize]);

  const handleMinimize = async () => {
    const appWindow = getCurrentWebviewWindow();
    await appWindow.minimize();
  };

  const handleToggleMaximize = async () => {
    const appWindow = getCurrentWebviewWindow();
    await appWindow.toggleMaximize();
  };

  const handleClose = async () => {
    const appWindow = getCurrentWebviewWindow();
    await appWindow.close();
  };

  useEffect(() => {
    if (!showClose) {
      return;
    }

    const appWindow = getCurrentWebviewWindow();
    if (appWindow.label === "main") {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") {
        return;
      }

      void handleClose();
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [showClose]);

  const handleDragRegionDoubleClick = () => {
    if (onDoubleClick) {
      onDoubleClick();
    } else if (showMaximize) {
      handleToggleMaximize();
    }
  };

  return (
    <div
      className={cn(
        "bg-background/95 supports-backdrop-filter:bg-background/60 border-border/40 flex items-center justify-between border-b backdrop-blur select-none",
        isAndroid() ? "h-14 px-4" : "h-8",
      )}
    >
      {/* Left: Title + Drag region */}
      <div
        data-tauri-drag-region={!isAndroid() ? true : undefined}
        onDoubleClick={handleDragRegionDoubleClick}
        className={cn(
          "flex grow items-center gap-2",
          isAndroid() ? "" : "pl-2",
        )}
      >
        {title && (
          <span
            className={cn(
              "font-medium",
              isAndroid()
                ? "text-base text-foreground"
                : "text-sm text-slate-400",
            )}
          >
            {title}
          </span>
        )}
        {leftActions}
      </div>

      {/* Right: Control buttons */}
      <div className={cn("flex items-center", isAndroid() && "gap-2")}>
        {rightActions}

        {!isAndroid() &&
          rightActions &&
          (showMinimize || showMaximize || showClose) && (
            <div className="bg-border/40 mx-1 h-4 w-px" />
          )}

        {!isAndroid() && showMinimize && (
          <button
            onClick={handleMinimize}
            className="title-bar-control"
            aria-label="最小化"
            tabIndex={-1}
          >
            <Minus className="h-4 w-4" />
          </button>
        )}

        {!isAndroid() && showMaximize && (
          <button
            onClick={handleToggleMaximize}
            className="title-bar-control"
            aria-label={isMaximized ? "还原" : "最大化"}
            tabIndex={-1}
          >
            {isMaximized ? (
              <Minimize2 className="h-4 w-4" />
            ) : (
              <Maximize2 className="h-4 w-4" />
            )}
          </button>
        )}

        {!isAndroid() && showClose && (
          <button
            onClick={handleClose}
            className="title-bar-control hover:bg-destructive hover:text-destructive-foreground"
            aria-label="关闭"
            tabIndex={-1}
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>
    </div>
  );
}
