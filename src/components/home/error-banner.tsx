import { useEffect, useState } from "react";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

interface ErrorBannerProps {
  message: string;
  onClose: () => void;
}

export function ErrorBanner({ message, onClose }: ErrorBannerProps) {
  const [displayMessage, setDisplayMessage] = useState("");
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (message) {
      setDisplayMessage(message);
      setVisible(true);
      return;
    }

    setVisible(false);
    const timer = window.setTimeout(() => {
      setDisplayMessage("");
    }, 300);
    return () => window.clearTimeout(timer);
  }, [message]);

  return (
    <div
      className={cn(
        "absolute left-1/2 z-50 w-[calc(100%-2rem)] max-w-xl -translate-x-1/2 transition-all duration-300 ease-out",
        visible
          ? "top-4 scale-100 opacity-100 pointer-events-auto"
          : "-top-20 scale-95 opacity-0 pointer-events-none",
      )}
    >
      {displayMessage && (
        <div className="border-red-500/30 dark:border-red-500/40 bg-card/95 flex items-start justify-between gap-3 rounded-xl border px-4 py-3.5 text-sm text-red-600 shadow-xl backdrop-blur-md dark:text-red-400">
          <div className="flex min-w-0 flex-1 items-start gap-2.5">
            <span className="mt-1.5 h-2 w-2 shrink-0 animate-pulse rounded-full bg-red-500" />
            <span className="flex-1 break-words whitespace-pre-wrap text-left font-medium select-text">
              {displayMessage}
            </span>
          </div>
          <button
            onClick={onClose}
            className="mt-0.5 shrink-0 rounded-lg p-1.5 text-red-600/70 transition-colors hover:bg-red-500/10 hover:text-red-600 dark:text-red-400/70 dark:hover:bg-red-500/20 dark:hover:text-red-400"
            aria-label="关闭错误提示"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      )}
    </div>
  );
}
