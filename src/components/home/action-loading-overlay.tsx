import appIconUrl from "../../../src-tauri/icons/icon.svg?url";
import type { AccountDto } from "@/lib/muc";
import { cn } from "@/lib/utils";
import type { RunningAction } from "./types";

interface ActionLoadingOverlayProps {
  runningAction: RunningAction | null;
  loginAccountId: string;
  accounts: AccountDto[];
}

export function ActionLoadingOverlay({
  runningAction,
  loginAccountId,
  accounts,
}: ActionLoadingOverlayProps) {
  if (!runningAction) {
    return null;
  }

  const targetAccount = accounts.find((account) => account.id === loginAccountId);
  const remarkName = targetAccount?.remarkName || targetAccount?.username;

  let config = {
    themeColor: "border-t-emerald-500",
    title: remarkName ? `正在登录 ${remarkName}...` : "正在登录校园网...",
    subtitle: "此过程可能需要一些时间，请稍候",
  };

  if (runningAction === "refresh") {
    config = {
      themeColor: "border-t-amber-500",
      title: "正在同步数据...",
      subtitle: "正在拉取最新的套餐流量与在线设备信息",
    };
  } else if (runningAction === "logout") {
    config = {
      themeColor: "border-t-red-500",
      title: "正在断开校园网...",
      subtitle: "正在向 Portal 认证服务器请求断开连接",
    };
  }

  return (
    <div className="absolute inset-0 z-50 flex animate-fade-in items-center justify-center bg-background/35 backdrop-blur-[4px] transition-all duration-300">
      <div className="bg-card/90 flex w-[90%] max-w-[260px] animate-scale-in-simple flex-col items-center gap-4 rounded-xl border border-border/40 p-6 shadow-xl backdrop-blur-md transition-all duration-300">
        <div className="relative flex h-14 w-14 items-center justify-center">
          <div
            className={cn(
              "absolute -inset-1.5 animate-spin rounded-full border border-muted/40",
              config.themeColor,
            )}
          />
          <img
            src={appIconUrl}
            alt="App Logo"
            className="h-10 w-10 shrink-0 animate-pulse rounded-lg"
          />
        </div>

        <div className="space-y-1 text-center">
          <h3 className="text-xs font-semibold tracking-wide text-foreground/90">
            {config.title}
          </h3>
          <p className="mx-auto max-w-[180px] text-[10px] leading-normal text-muted-foreground">
            {config.subtitle}
          </p>
        </div>
      </div>
    </div>
  );
}
