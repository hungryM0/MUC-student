import { CircleGauge, Power } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { AppSnapshotDto } from "@/lib/muc";
import { cn } from "@/lib/utils";
import {
  buildPoolUsage,
  formatLocalLoginTime,
  formatTrafficAmount,
  trafficProgressClasses,
} from "./utils";

interface OverviewSectionProps {
  snapshot: AppSnapshotDto | null;
  canLogoutLocalDevice: boolean;
  isBusy: boolean;
  onLogoutLocalDevice: () => void;
}

export function OverviewSection({
  snapshot,
  canLogoutLocalDevice,
  isBusy,
  onLogoutLocalDevice,
}: OverviewSectionProps) {
  const poolUsage = buildPoolUsage(snapshot?.accounts ?? []);
  const hasUnlimitedPlan = poolUsage.hasUnlimitedPlan;
  const progressPercent = Math.round(poolUsage.totalProgress);
  const safeProgress = Math.min(100, Math.max(0, progressPercent));
  const currentOnlineRemark = snapshot?.currentOnlineAccountId
    ? snapshot.accounts.find(
        (account) => account.id === snapshot.currentOnlineAccountId,
      )?.remarkName || "未知账号"
    : "无在线设备";

  return (
    <Card className="border-border bg-background/95 rounded-xl backdrop-blur-sm">
      <CardHeader className="border-b border-border/40 pb-3">
        <CardTitle className="flex items-center justify-between text-base font-semibold">
          <div className="flex items-center gap-2">
            <CircleGauge className="h-4.5 w-4.5 text-emerald-500" />
            号池概览
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-5 pt-4">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-muted-foreground">
              流量池已用
            </span>
            <span
              className={cn(
                "text-sm font-bold",
                hasUnlimitedPlan
                  ? "unlimited-plan-text"
                  : trafficProgressClasses(safeProgress).text,
              )}
            >
              {hasUnlimitedPlan ? "不限流量" : `${safeProgress}%`}
            </span>
          </div>
          <div className="bg-muted h-2 overflow-hidden rounded-full">
            <div
              className={cn(
                "h-full rounded-full transition-[width] duration-500",
                hasUnlimitedPlan
                  ? "unlimited-plan-bar"
                  : trafficProgressClasses(safeProgress).bar,
              )}
              style={{ width: `${hasUnlimitedPlan ? 100 : safeProgress}%` }}
            />
          </div>
          <div className="mt-2 flex flex-col gap-1">
            <div className="text-xl font-bold tracking-tight">
              {poolUsage.hasSnapshot
                ? hasUnlimitedPlan
                  ? `${formatTrafficAmount(poolUsage.totalUsed)} / `
                  : `${formatTrafficAmount(poolUsage.totalUsed)} / ${formatTrafficAmount(poolUsage.totalQuota)}`
                : "-"}
              {poolUsage.hasSnapshot && hasUnlimitedPlan && (
                <span className="unlimited-plan-text">不限流量</span>
              )}
            </div>
            <div className="truncate text-xs text-muted-foreground">
              {hasUnlimitedPlan
                ? "含不限流量账号"
                : (snapshot?.poolQuota.productBalanceText ?? "-")}
            </div>
            <div className="mt-1 whitespace-pre-wrap rounded border border-border/20 bg-muted/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground">
              {snapshot?.poolQuota.includedPackageText || "套餐信息为空"}
            </div>
          </div>
        </div>

        <div className="my-3 border-t border-border/40" />

        <div className="space-y-3.5">
          <div className="flex items-center justify-between gap-3">
            <span className="text-xs text-muted-foreground">当前在线</span>
            <span className="max-w-[70%] truncate text-xs font-medium">
              {currentOnlineRemark}
            </span>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">最近登录</span>
            <span className="font-mono text-xs text-muted-foreground">
              {formatLocalLoginTime(snapshot?.loginState.lastLoginTime)}
            </span>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">已配置账号</span>
            <span className="text-xs font-medium">
              {snapshot?.accounts.length || 0}
            </span>
          </div>
        </div>

        {canLogoutLocalDevice && (
          <div className="pt-2">
            <Button
              variant="ghost"
              size="sm"
              disabled={isBusy}
              onClick={onLogoutLocalDevice}
              className="h-8 w-full gap-1.5 border border-border/30 text-xs text-muted-foreground transition-all hover:border-destructive/30 hover:bg-destructive/10 hover:text-destructive"
            >
              <Power className="h-3 w-3" />
              断开校园网
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
