import { invoke } from "@tauri-apps/api/core";

export type NetworkStatus = {
  isOnline: boolean;
  statusText: string;
  ip: string;
  checkedAt: string;
};

export type AccountTrafficSnapshot = {
  accountId: string;
  usedTrafficText: string;
  productBalanceText: string;
  includedPackageText: string;
  onlineDeviceCountText: string;
  packageText: string;
  statusText: string;
  detailText: string;
  queriedAt: string;
  progressPercent: number | null;
};

export type AccountDto = {
  id: string;
  remarkName: string;
  username: string;
  snapshot: AccountTrafficSnapshot | null;
  isCurrentOnline: boolean;
  canLogoutLocalDevice: boolean;
};

export type AppSnapshotDto = {
  network: NetworkStatus;
  accounts: AccountDto[];
  selectedAccountId: string;
  currentOnlineAccountId: string;
  poolQuota: {
    usedTrafficText: string;
    productBalanceText: string;
    includedPackageText: string;
    progressPercent: number | null;
  };
  loginState: {
    running: boolean;
    lastLoginTime: string | null;
    resultText: string;
    message: string;
  };
  refreshState: {
    running: boolean;
    lastQuotaRefreshTime: string | null;
  };
  preferences: {
    minimizeToTrayOnClose: boolean;
    launchOnStartup: boolean;
    autoSwitchAccountOnTrafficExhausted: boolean;
  };
};

export type AppErrorDto = {
  code: string;
  message: string;
  detail: string;
};

export async function bootstrapApp() {
  return invoke<AppSnapshotDto>("bootstrap_app");
}

export async function getAppSnapshot() {
  return invoke<AppSnapshotDto>("get_app_snapshot");
}

export async function selectAccount(accountId: string) {
  return invoke<AppSnapshotDto>("select_account", { accountId });
}

export async function loginSelectedAccount() {
  return invoke<AppSnapshotDto>("login_selected_account");
}

export async function refreshDashboard() {
  return invoke<AppSnapshotDto>("refresh_dashboard");
}

export async function logoutLocalDevice() {
  return invoke<AppSnapshotDto>("logout_local_device");
}

export function readErrorMessage(error: unknown) {
  if (typeof error === "object" && error && "message" in error) {
    return String((error as AppErrorDto).message);
  }
  return String(error);
}
