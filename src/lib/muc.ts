import { invoke } from "@tauri-apps/api/core";

export const FREE_PRODUCT_QUOTA_GB = 70;

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
  packageTotalText: string;
  packageAvailableText: string;
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

export type AccountPoolImportResultDto = {
  snapshot: AppSnapshotDto;
  importedCount: number;
  overwrittenCount: number;
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

export async function addAccount(
  remarkName: string,
  username: string,
  password: string,
) {
  return invoke<AppSnapshotDto>("add_account", {
    remarkName,
    username,
    password,
  });
}

export async function updateAccount(
  accountId: string,
  remarkName: string,
  username: string,
  password?: string,
) {
  return invoke<AppSnapshotDto>("update_account", {
    accountId,
    remarkName,
    username,
    password: password?.trim() ? password : null,
  });
}

export async function deleteAccount(accountId: string) {
  return invoke<AppSnapshotDto>("delete_account", { accountId });
}

export async function exportAccountPool(passphrase: string) {
  return invoke<string>("export_account_pool", { passphrase });
}

export async function importAccountPool(code: string, passphrase: string) {
  return invoke<AccountPoolImportResultDto>("import_account_pool", {
    code,
    passphrase,
  });
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

export async function updatePreferences(
  preferences: AppSnapshotDto["preferences"],
) {
  return invoke<AppSnapshotDto>("update_preferences", {
    minimizeToTrayOnClose: preferences.minimizeToTrayOnClose,
    launchOnStartup: preferences.launchOnStartup,
    autoSwitchAccountOnTrafficExhausted:
      preferences.autoSwitchAccountOnTrafficExhausted,
  });
}

export function readErrorMessage(error: unknown) {
  if (typeof error === "object" && error && "message" in error) {
    return String((error as AppErrorDto).message);
  }
  return String(error);
}
