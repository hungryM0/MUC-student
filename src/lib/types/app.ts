export type NetworkStatus = {
  isOnline: boolean;
  statusText: string;
  ip: string;
  checkedAt: string;
};

export type OnlineDeviceRecord = {
  ip: string;
  deviceId: string;
  logoutPath: string;
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
  onlineDevices: OnlineDeviceRecord[];
  matchedLocalIpDevice: OnlineDeviceRecord | null;
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

export type PreferenceDto = {
  minimizeToTrayOnClose: boolean;
  launchOnStartup: boolean;
  autoSwitchAccountOnTrafficExhausted: boolean;
};

export type LoginStateDto = {
  running: boolean;
  lastLoginTime: string | null;
  resultText: string;
  message: string;
};

export type RefreshStateDto = {
  running: boolean;
  lastQuotaRefreshTime: string | null;
};

export type PoolQuotaDto = {
  usedTrafficText: string;
  productBalanceText: string;
  includedPackageText: string;
  progressPercent: number | null;
};

export type LogItemDto = {
  timestamp: string;
  level: string;
  message: string;
};

export type AppSnapshotDto = {
  network: NetworkStatus;
  accounts: AccountDto[];
  selectedAccountId: string;
  currentOnlineAccountId: string;
  poolQuota: PoolQuotaDto;
  loginState: LoginStateDto;
  refreshState: RefreshStateDto;
  preferences: PreferenceDto;
  logs: LogItemDto[];
};

export type AppErrorDto = {
  code: string;
  message: string;
  detail: string;
};

export type AccountInput = {
  remarkName: string;
  username: string;
  password: string;
};

export type AccountUpdateInput = {
  accountId: string;
  remarkName: string;
  username: string;
  password?: string | null;
};

export type PreferenceInput = PreferenceDto;

export type UiState = {
  activePage: 'home' | 'status' | 'settings';
  loadingMessage: string;
  error: AppErrorDto | null;
  sortMode: 'default' | 'remainingDesc' | 'nameAsc';
};

export type DialogState =
  | { type: 'none' }
  | { type: 'account'; mode: 'create'; accountId?: undefined }
  | { type: 'account'; mode: 'edit'; accountId: string }
  | { type: 'confirmDelete'; accountId: string }
  | { type: 'confirmLogout'; accountId: string };
