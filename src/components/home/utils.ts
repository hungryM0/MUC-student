import { FREE_PRODUCT_QUOTA_GB, type AccountDto } from "@/lib/muc";

export function trafficProgressClasses(percent: number, isOnline = true) {
  const colors =
    percent >= 100
      ? { bar: "bg-red-700", barOff: "bg-red-700/60", text: "text-red-700" }
      : percent >= 90
        ? { bar: "bg-red-500", barOff: "bg-red-500/60", text: "text-red-500" }
        : percent >= 80
          ? {
              bar: "bg-orange-500",
              barOff: "bg-orange-500/60",
              text: "text-orange-500",
            }
          : percent >= 70
            ? {
                bar: "bg-yellow-500",
                barOff: "bg-yellow-500/60",
                text: "text-yellow-500",
              }
            : {
                bar: "bg-emerald-500",
                barOff: "bg-emerald-500/60",
                text: "text-emerald-500",
              };

  return { bar: isOnline ? colors.bar : colors.barOff, text: colors.text };
}

export function parseTrafficValue(text?: string | null) {
  if (!text) {
    return 0;
  }

  const match = text.trim().match(/([0-9]+(?:\.[0-9]+)?)/);
  return match ? Number.parseFloat(match[1]) : 0;
}

export function formatTrafficAmount(value: number, digits = 3) {
  return `${value.toFixed(digits)}G`;
}

export function formatLocalLoginTime(value?: string | null) {
  if (!value) {
    return "无记录";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "无记录";
  }

  const parts = new Intl.DateTimeFormat("zh-CN", {
    timeZone: "Asia/Shanghai",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).formatToParts(date);
  const getPart = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((part) => part.type === type)?.value ?? "00";

  return `${getPart("year")}-${getPart("month")}-${getPart("day")} ${getPart("hour")}:${getPart("minute")}:${getPart("second")}`;
}

export function formatSnapshotSyncText(snapshot: AccountDto["snapshot"]) {
  if (!snapshot) {
    return "未查询";
  }
  if (
    snapshot.statusText === "查询中..." ||
    snapshot.statusText === "查询失败"
  ) {
    return snapshot.statusText;
  }

  const queriedAt = new Date(snapshot.queriedAt);
  if (Number.isNaN(queriedAt.getTime())) {
    return snapshot.statusText || "未查询";
  }

  const elapsedSeconds = Math.max(
    0,
    Math.floor((Date.now() - queriedAt.getTime()) / 1000),
  );
  if (elapsedSeconds < 60) {
    return "刚刚同步";
  }

  const elapsedMinutes = Math.floor(elapsedSeconds / 60);
  if (elapsedMinutes < 60) {
    return `${elapsedMinutes} 分钟前同步`;
  }

  const elapsedHours = Math.floor(elapsedMinutes / 60);
  if (elapsedHours < 24) {
    return `${elapsedHours} 小时前同步`;
  }

  return `${Math.floor(elapsedHours / 24)} 天前同步`;
}

export function buildAccountUsage(account: AccountDto) {
  const snapshot = account.snapshot;
  const isUnlimitedPlan = snapshot?.isUnlimitedPlan ?? false;
  const freeUsed = parseTrafficValue(snapshot?.usedTrafficText);
  const packageTotal = parseTrafficValue(snapshot?.packageTotalText);
  const packageAvailable = parseTrafficValue(snapshot?.packageAvailableText);
  const packageUsed = Math.max(0, packageTotal - packageAvailable);
  const totalUsed = freeUsed + packageUsed;
  const totalQuota = FREE_PRODUCT_QUOTA_GB + packageTotal;
  const freeProgress =
    FREE_PRODUCT_QUOTA_GB > 0
      ? Math.min(100, Math.max(0, (freeUsed / FREE_PRODUCT_QUOTA_GB) * 100))
      : 0;
  const packageProgress =
    packageTotal > 0
      ? Math.min(100, Math.max(0, (packageUsed / packageTotal) * 100))
      : 0;
  const totalProgress =
    totalQuota > 0
      ? Math.min(100, Math.max(0, (totalUsed / totalQuota) * 100))
      : 0;

  return {
    isUnlimitedPlan,
    freeUsed,
    totalUsed,
    totalQuota,
    packageTotal,
    packageUsed,
    freeQuota: FREE_PRODUCT_QUOTA_GB,
    freeProgress,
    packageProgress,
    totalProgress,
  };
}

export function buildPoolUsage(accounts: AccountDto[]) {
  const initial = {
    freeUsed: 0,
    totalUsed: 0,
    totalQuota: 0,
    hasUnlimitedPlan: false,
    hasSnapshot: false,
  };

  const usage = accounts.reduce((acc, account) => {
    if (!account.snapshot) {
      return acc;
    }

    const accountUsage = buildAccountUsage(account);
    return {
      freeUsed: acc.freeUsed + accountUsage.freeUsed,
      totalUsed: acc.totalUsed + accountUsage.totalUsed,
      totalQuota: acc.totalQuota + accountUsage.totalQuota,
      hasUnlimitedPlan: acc.hasUnlimitedPlan || accountUsage.isUnlimitedPlan,
      hasSnapshot: true,
    };
  }, initial);

  const totalProgress =
    usage.totalQuota > 0
      ? Math.min(100, Math.max(0, (usage.totalUsed / usage.totalQuota) * 100))
      : 0;

  return {
    ...usage,
    totalProgress,
  };
}
