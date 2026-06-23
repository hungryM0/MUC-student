import { ThemeProvider } from "@/components/theme-provider";
import { cn } from "@/lib/utils";
import { type ReactNode } from "react";

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
  return (
    <ThemeProvider defaultTheme="system" storageKey="tauri-ui-theme">
      <div
        className={cn(
          "bg-background flex h-screen w-screen flex-col overflow-hidden",
          className,
        )}
      >
        {titleBar}
        <main className={contentClassName}>{children}</main>
      </div>
    </ThemeProvider>
  );
}
