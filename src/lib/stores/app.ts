import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { get, writable } from 'svelte/store';
import type {
  AccountDto,
  AccountInput,
  AccountUpdateInput,
  AppErrorDto,
  AppSnapshotDto,
  DialogState,
  LogItemDto,
  PreferenceInput,
  UiState
} from '$lib/types/app';

export const appSnapshot = writable<AppSnapshotDto | null>(null);
export const uiState = writable<UiState>({
  activePage: 'home',
  loadingMessage: '',
  error: null,
  sortMode: 'default'
});
export const dialogState = writable<DialogState>({ type: 'none' });

let eventListenersReady = false;
let windowCloseReady = false;
let unlisteners: UnlistenFn[] = [];

export function setActivePage(activePage: UiState['activePage']) {
  uiState.update((state) => ({ ...state, activePage }));
}

export function setSortMode(sortMode: UiState['sortMode']) {
  uiState.update((state) => ({ ...state, sortMode }));
}

export function clearError() {
  uiState.update((state) => ({ ...state, error: null }));
}

export function openCreateAccountDialog() {
  dialogState.set({ type: 'account', mode: 'create' });
}

export function openEditAccountDialog(accountId: string) {
  dialogState.set({ type: 'account', mode: 'edit', accountId });
}

export function openDeleteConfirm(accountId: string) {
  dialogState.set({ type: 'confirmDelete', accountId });
}

export function openLogoutConfirm(accountId: string) {
  dialogState.set({ type: 'confirmLogout', accountId });
}

export function closeDialog() {
  dialogState.set({ type: 'none' });
}

export async function initializeTauriBridge() {
  await initializeAppEvents();
  await initializeWindowCloseHandler();
}

export async function bootstrapApp() {
  await initializeTauriBridge();
  return command<AppSnapshotDto>('bootstrapApp', undefined, '启动中...');
}

export function getAppSnapshot() {
  return command<AppSnapshotDto>('getAppSnapshot');
}

export function selectAccount(accountId: string) {
  return command<AppSnapshotDto>('selectAccount', { accountId }, '正在切换账号...');
}

export function createAccount(input: AccountInput) {
  return command<AppSnapshotDto>('createAccount', { input }, '正在保存账号...');
}

export function updateAccount(input: AccountUpdateInput) {
  return command<AppSnapshotDto>('updateAccount', { input }, '正在保存账号...');
}

export function deleteAccount(accountId: string) {
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
  return command<AppSnapshotDto>('updatePreferences', { input }, '正在保存设置...');
}

export function findAccount(snapshot: AppSnapshotDto | null, accountId: string): AccountDto | null {
  return snapshot?.accounts.find((account) => account.id === accountId) ?? null;
}

async function initializeAppEvents() {
  if (eventListenersReady) return;
  eventListenersReady = true;

  unlisteners = [
    await listen<AppSnapshotDto>('app://state-updated', (event) => {
      appSnapshot.set(event.payload);
    }),
    await listen<LogItemDto>('app://log-appended', (event) => {
      appendLogIfNeeded(event.payload);
    }),
    await listen<string>('app://task-started', (event) => {
      uiState.update((state) => ({ ...state, loadingMessage: taskLabel(event.payload) }));
    }),
    await listen<string>('app://task-finished', () => {
      uiState.update((state) => ({ ...state, loadingMessage: '' }));
    })
  ];
}

async function initializeWindowCloseHandler() {
  if (windowCloseReady) return;
  windowCloseReady = true;

  const window = getCurrentWindow();
  const unlisten = await window.onCloseRequested(async (event) => {
    const snapshot = get(appSnapshot);
    if (!snapshot?.preferences.minimizeToTrayOnClose) return;
    event.preventDefault();
    await window.hide();
  });
  unlisteners.push(unlisten);
}

async function command<T>(name: string, args?: Record<string, unknown>, loadingMessage = ''): Promise<T> {
  uiState.update((state) => ({ ...state, error: null, loadingMessage }));
  try {
    const result = await invoke<T>(name, args);
    if (isSnapshot(result)) {
      appSnapshot.set(result);
    }
    return result;
  } catch (error) {
    const appError = toAppError(error);
    uiState.update((state) => ({ ...state, error: appError }));
    throw appError;
  } finally {
    uiState.update((state) => ({ ...state, loadingMessage: '' }));
  }
}

function appendLogIfNeeded(log: LogItemDto) {
  appSnapshot.update((snapshot) => {
    if (!snapshot) return snapshot;
    const exists = snapshot.logs.some(
      (item) => item.timestamp === log.timestamp && item.level === log.level && item.message === log.message
    );
    if (exists) return snapshot;
    return { ...snapshot, logs: [...snapshot.logs, log].slice(-500) };
  });
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

export function disposeTauriBridge() {
  for (const unlisten of unlisteners) unlisten();
  unlisteners = [];
  eventListenersReady = false;
  windowCloseReady = false;
}
