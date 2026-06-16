import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AccountDto,
  AccountInput,
  AccountUpdateInput,
  AppErrorDto,
  AppSnapshotDto,
  DialogState,
  PreferenceInput,
  UiState
} from '$lib/types/app';

export type AppStoreState = {
  appSnapshot: AppSnapshotDto | null;
  uiState: UiState;
  dialogState: DialogState;
};

let state: AppStoreState = {
  appSnapshot: null,
  uiState: {
    activePage: 'home',
    loadingMessage: '',
    error: null,
    sortMode: 'default'
  },
  dialogState: { type: 'none' }
};

const subscribers = new Set<() => void>();
let eventListenersReady = false;
let unlisteners: UnlistenFn[] = [];

export function subscribeAppStore(listener: () => void) {
  subscribers.add(listener);
  return () => subscribers.delete(listener);
}

export function getAppStoreSnapshot() {
  return state;
}

export function setActivePage(activePage: UiState['activePage']) {
  setUiState({ activePage });
}

export function setSortMode(sortMode: UiState['sortMode']) {
  setUiState({ sortMode });
}

export function clearError() {
  setUiState({ error: null });
}

export function openCreateAccountDialog() {
  setDialogState({ type: 'account', mode: 'create' });
}

export function openEditAccountDialog(accountId: string) {
  setDialogState({ type: 'account', mode: 'edit', accountId });
}

export function openDeleteConfirm(accountId: string) {
  setDialogState({ type: 'confirmDelete', accountId });
}

export function openLogoutConfirm(accountId: string) {
  setDialogState({ type: 'confirmLogout', accountId });
}

export function closeDialog() {
  setDialogState({ type: 'none' });
}

export async function initializeTauriBridge() {
  await initializeAppEvents();
}

export async function bootstrapApp() {
  await initializeTauriBridge();
  if (!isTauriRuntime()) {
    const snapshot = createBrowserPreviewSnapshot();
    setSnapshot(snapshot);
    return snapshot;
  }
  return command<AppSnapshotDto>('bootstrapApp', undefined, '启动中...');
}

export function getBackendSnapshot() {
  return command<AppSnapshotDto>('getAppSnapshot');
}

export function selectAccount(accountId: string) {
  if (!isTauriRuntime() && state.appSnapshot) {
    const snapshot = { ...state.appSnapshot, selectedAccountId: accountId };
    setSnapshot(snapshot);
    return Promise.resolve(snapshot);
  }
  return command<AppSnapshotDto>('selectAccount', { accountId }, '正在切换账号...');
}

export function createAccount(input: AccountInput) {
  if (!isTauriRuntime() && state.appSnapshot) {
    const id = crypto.randomUUID();
    const snapshot = {
      ...state.appSnapshot,
      selectedAccountId: id,
      accounts: [
        ...state.appSnapshot.accounts,
        {
          id,
          remarkName: input.remarkName,
          username: input.username,
          snapshot: null,
          isCurrentOnline: false,
          canLogoutLocalDevice: false
        }
      ]
    };
    setSnapshot(snapshot);
    return Promise.resolve(snapshot);
  }
  return command<AppSnapshotDto>('createAccount', { input }, '正在保存账号...');
}

export function updateAccount(input: AccountUpdateInput) {
  if (!isTauriRuntime() && state.appSnapshot) {
    const snapshot = {
      ...state.appSnapshot,
      accounts: state.appSnapshot.accounts.map((account) =>
        account.id === input.accountId ? { ...account, remarkName: input.remarkName, username: input.username } : account
      )
    };
    setSnapshot(snapshot);
    return Promise.resolve(snapshot);
  }
  return command<AppSnapshotDto>('updateAccount', { input }, '正在保存账号...');
}

export function deleteAccount(accountId: string) {
  if (!isTauriRuntime() && state.appSnapshot) {
    const accounts = state.appSnapshot.accounts.filter((account) => account.id !== accountId);
    const snapshot = {
      ...state.appSnapshot,
      accounts,
      selectedAccountId: state.appSnapshot.selectedAccountId === accountId ? accounts[0]?.id ?? '' : state.appSnapshot.selectedAccountId
    };
    setSnapshot(snapshot);
    return Promise.resolve(snapshot);
  }
  return command<AppSnapshotDto>('deleteAccount', { accountId }, '正在删除账号...');
}

export function loginSelectedAccount() {
  return command<AppSnapshotDto>('loginSelectedAccount', undefined, '正在登录...');
}

export function refreshDashboard() {
  return command<AppSnapshotDto>('refreshDashboard', undefined, '正在刷新状态...');
}

export function logoutLocalDevice() {
  return command<AppSnapshotDto>('logoutLocalDevice', undefined, '正在下线本机...');
}

export function updatePreferences(input: PreferenceInput) {
  if (!isTauriRuntime() && state.appSnapshot) {
    const snapshot = { ...state.appSnapshot, preferences: input };
    setSnapshot(snapshot);
    return Promise.resolve(snapshot);
  }
  return command<AppSnapshotDto>('updatePreferences', { input }, '正在保存设置...');
}

export function findAccount(snapshot: AppSnapshotDto | null, accountId: string): AccountDto | null {
  return snapshot?.accounts.find((account) => account.id === accountId) ?? null;
}

function setSnapshot(appSnapshot: AppSnapshotDto | null) {
  state = { ...state, appSnapshot };
  emit();
}

function setUiState(patch: Partial<UiState>) {
  state = { ...state, uiState: { ...state.uiState, ...patch } };
  emit();
}

function setDialogState(dialogState: DialogState) {
  state = { ...state, dialogState };
  emit();
}

function emit() {
  for (const subscriber of subscribers) subscriber();
}

async function initializeAppEvents() {
  if (!isTauriRuntime()) return;
  if (eventListenersReady) return;
  eventListenersReady = true;

  unlisteners = [
    await listen<AppSnapshotDto>('app://state-updated', (event) => {
      setSnapshot(event.payload);
    }),
    await listen<string>('app://task-started', (event) => {
      setUiState({ loadingMessage: taskLabel(event.payload) });
    }),
    await listen<string>('app://task-finished', () => {
      setUiState({ loadingMessage: '' });
    })
  ];
}

async function command<T>(name: string, args?: Record<string, unknown>, loadingMessage = ''): Promise<T> {
  setUiState({ error: null, loadingMessage });
  try {
    const result = await invoke<T>(name, args);
    if (isSnapshot(result)) {
      setSnapshot(result);
    }
    return result;
  } catch (error) {
    const appError = toAppError(error);
    setUiState({ error: appError });
    throw appError;
  } finally {
    setUiState({ loadingMessage: '' });
  }
}

function isSnapshot(value: unknown): value is AppSnapshotDto {
  return typeof value === 'object' && value !== null && 'accounts' in value && 'preferences' in value;
}

function toAppError(error: unknown): AppErrorDto {
  if (typeof error === 'object' && error !== null && 'code' in error && 'message' in error) {
    const value = error as Partial<AppErrorDto>;
    return {
      code: String(value.code ?? 'UNKNOWN_ERROR'),
      message: String(value.message ?? '未知错误'),
      detail: String(value.detail ?? value.message ?? '')
    };
  }
  return {
    code: 'UNKNOWN_ERROR',
    message: typeof error === 'string' ? error : '操作失败',
    detail: typeof error === 'string' ? error : JSON.stringify(error)
  };
}

function taskLabel(task: string) {
  if (task === 'login') return '正在登录...';
  if (task === 'refresh') return '正在刷新状态...';
  if (task === 'logout') return '正在下线本机...';
  return '正在处理...';
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

function createBrowserPreviewSnapshot(): AppSnapshotDto {
  const now = new Date().toISOString();
  return {
    network: {
      isOnline: true,
      statusText: '预览在线',
      ip: '10.18.24.16',
      checkedAt: now
    },
    selectedAccountId: 'acc-1',
    currentOnlineAccountId: 'acc-1',
    accounts: [
      {
        id: 'acc-1',
        remarkName: '主力账号',
        username: '20260001',
        isCurrentOnline: true,
        canLogoutLocalDevice: true,
        snapshot: {
          accountId: 'acc-1',
          usedTrafficText: '18.4 GB',
          productBalanceText: '30 GB',
          includedPackageText: '月包',
          onlineDeviceCountText: '2',
          packageText: '校园网套餐',
          statusText: '正常',
          detailText: '',
          queriedAt: now,
          onlineDevices: [],
          matchedLocalIpDevice: null,
          progressPercent: 61.3
        }
      },
      {
        id: 'acc-2',
        remarkName: '备用账号',
        username: '20260002',
        isCurrentOnline: false,
        canLogoutLocalDevice: false,
        snapshot: {
          accountId: 'acc-2',
          usedTrafficText: '4.2 GB',
          productBalanceText: '30 GB',
          includedPackageText: '月包',
          onlineDeviceCountText: '0',
          packageText: '校园网套餐',
          statusText: '正常',
          detailText: '',
          queriedAt: now,
          onlineDevices: [],
          matchedLocalIpDevice: null,
          progressPercent: 14
        }
      },
      {
        id: 'acc-3',
        remarkName: '已用尽',
        username: '20260003',
        isCurrentOnline: false,
        canLogoutLocalDevice: false,
        snapshot: {
          accountId: 'acc-3',
          usedTrafficText: '30 GB',
          productBalanceText: '30 GB',
          includedPackageText: '月包',
          onlineDeviceCountText: '0',
          packageText: '校园网套餐',
          statusText: '已用尽',
          detailText: '',
          queriedAt: now,
          onlineDevices: [],
          matchedLocalIpDevice: null,
          progressPercent: 100
        }
      }
    ],
    poolQuota: {
      usedTrafficText: '52.6 GB',
      productBalanceText: '90 GB',
      includedPackageText: '三账号',
      progressPercent: 58.4
    },
    loginState: {
      running: false,
      lastLoginTime: now,
      resultText: '登录成功',
      message: ''
    },
    refreshState: {
      running: false,
      lastQuotaRefreshTime: now
    },
    preferences: {
      minimizeToTrayOnClose: true,
      launchOnStartup: false,
      autoSwitchAccountOnTrafficExhausted: true
    }
  };
}

export function disposeTauriBridge() {
  for (const unlisten of unlisteners) unlisten();
  unlisteners = [];
  eventListenersReady = false;
}
