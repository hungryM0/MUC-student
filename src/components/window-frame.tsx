import { ThemeProvider } from "@/components/theme-provider";
import { cn } from "@/lib/utils";
import { type ReactNode, useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

type WindowFrameProps = {
  titleBar: ReactNode;
  children: ReactNode;
  className?: string;
  contentClassName?: string;
};

export function WindowFrame({
  titleBar,
  children,
  className,
  contentClassName,
}: WindowFrameProps) {
  const [isMain, setIsMain] = useState(true);

  useEffect(() => {
    try {
      const label = getCurrentWebviewWindow().label;
      setIsMain(label === "main");
    } catch {
      setIsMain(true);
    }
  }, []);

  return (
    <ThemeProvider defaultTheme="system" storageKey="tauri-ui-theme">
      <div
        className={cn(
          "bg-background text-foreground flex h-screen w-screen flex-col overflow-hidden",
          isMain
            ? "border border-border/70"
            : "rounded-xl border border-border/80 shadow-2xl",
          className,
        )}
      >
        {titleBar}
        <main className={contentClassName}>{children}</main>
      </div>
    </ThemeProvider>
  );
}
