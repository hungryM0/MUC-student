import {
  Button,
  Card,
  FluentProvider,
  Spinner,
  Text,
  webDarkTheme,
  webLightTheme,
  type Theme
} from '@fluentui/react-components';
import {
  HomeRegular,
  SettingsRegular,
  WeatherMoonRegular,
  WeatherSunnyRegular,
  Wifi1Regular
} from '@fluentui/react-icons';
import { useEffect, useMemo, useState, type ReactElement } from 'react';
import { AccountDialog } from '$lib/features/accounts/AccountDialog';
import { HomePanel } from '$lib/features/home/HomePanel';
import { ConfirmDialog } from '$lib/features/shared/ConfirmDialog';
import { iconSize, useShellStyles } from '$lib/features/shared/layout';
import { SettingsPanel } from '$lib/features/settings/SettingsPanel';
import { useAppStore } from '$lib/hooks/use-app-store';
import {
  bootstrapApp,
  closeDialog,
  createAccount,
  deleteAccount,
  disposeTauriBridge,
  findAccount,
  loginSelectedAccount,
  logoutLocalDevice,
  openCreateAccountDialog,
  openDeleteConfirm,
  openEditAccountDialog,
  openLogoutConfirm,
  refreshDashboard,
  selectAccount,
  setActivePage,
  setSortMode,
  updateAccount,
  updatePreferences
} from '$lib/stores/app';
import type { AccountInput, AccountUpdateInput, PreferenceInput, UiState } from '$lib/types/app';

type ThemeMode = 'dark' | 'light';

function systemTheme(): ThemeMode {
  if (typeof window !== 'undefined' && window.matchMedia?.('(prefers-color-scheme: light)').matches) {
    return 'light';
  }
  return 'dark';
}

export default function App() {
  const styles = useShellStyles();
  const { appSnapshot, uiState, dialogState } = useAppStore();
  const [themeMode, setThemeMode] = useState<ThemeMode>(systemTheme);

  useEffect(() => {
    bootstrapApp().catch(() => undefined);
    return () => disposeTauriBridge();
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = themeMode;
    document.documentElement.style.background = 'transparent';
    document.body.style.background = 'transparent';
  }, [themeMode]);

  const theme: Theme = themeMode === 'dark' ? webDarkTheme : webLightTheme;
  const busy = Boolean(uiState.loadingMessage || appSnapshot?.loginState.running || appSnapshot?.refreshState.running);
  const pageTitle = useMemo(() => (uiState.activePage === 'settings' ? '设置' : 'MUC-student 状态'), [uiState.activePage]);

  const submitAccount = async (input: AccountInput | AccountUpdateInput) => {
    try {
      if ('accountId' in input) {
        await updateAccount(input);
      } else {
        await createAccount(input);
      }
      closeDialog();
    } catch {
      return;
    }
  };

  const confirmDelete = async (accountId: string) => {
    try {
      await deleteAccount(accountId);
      closeDialog();
    } catch {
      return;
    }
  };

  const confirmLogout = async () => {
    try {
      await logoutLocalDevice();
      closeDialog();
    } catch {
      return;
    }
  };

  const savePreferences = async (preferences: PreferenceInput) => {
    try {
      await updatePreferences(preferences);
    } catch {
      return;
    }
  };

  return (
    <FluentProvider theme={theme} className={styles.provider} style={{ background: 'transparent' }}>
      {!appSnapshot ? (
        <StartupView loadingMessage={uiState.loadingMessage} error={uiState.error?.message || ''} detail={uiState.error?.detail || ''} />
      ) : (
        <div className={styles.frame}>
          <aside className={styles.sidebar} data-tauri-drag-region>
            <div className={styles.brand}>MUC-student</div>
            <NavButton active={uiState.activePage === 'home'} icon={<HomeRegular style={iconSize} />} label="主页" page="home" />
            <NavButton active={uiState.activePage === 'settings'} icon={<SettingsRegular style={iconSize} />} label="设置" page="settings" />
            <div className={styles.sidebarSpacer} />
            <Button
              appearance="subtle"
              className={styles.navButton}
              icon={themeMode === 'dark' ? <WeatherSunnyRegular style={iconSize} /> : <WeatherMoonRegular style={iconSize} />}
              onClick={() => setThemeMode((value) => (value === 'dark' ? 'light' : 'dark'))}
            >
              Day Mode
            </Button>
          </aside>

          <main className={styles.page}>
            <header className={styles.header} data-tauri-drag-region>
              <h1 className={styles.title}>{pageTitle}</h1>
              <div className={styles.toolbar}>
                <Text className={appSnapshot.network.isOnline ? styles.success : styles.danger}>
                  {appSnapshot.network.statusText}
                </Text>
                <Text className={styles.muted}>IP: {appSnapshot.network.ip || 'unknown'}</Text>
                {uiState.loadingMessage && <Text className={styles.brandText}>{uiState.loadingMessage}</Text>}
              </div>
            </header>

            <section className={styles.content}>
              {uiState.error && (
                <Card appearance="filled" className={styles.compactCard}>
                  <Text weight="semibold" className={styles.danger}>
                    {uiState.error.message}
                  </Text>
                  {uiState.error.detail && <Text className={styles.muted}>{uiState.error.detail}</Text>}
                </Card>
              )}

              {uiState.activePage === 'home' && (
                <HomePanel
                  snapshot={appSnapshot}
                  sortMode={uiState.sortMode}
                  busy={busy}
                  onSelectAccount={(accountId) => selectAccount(accountId).catch(() => undefined)}
                  onLogin={() => loginSelectedAccount().catch(() => undefined)}
                  onSortMode={setSortMode}
                  onAdd={openCreateAccountDialog}
                  onEdit={openEditAccountDialog}
                  onDelete={openDeleteConfirm}
                  onLogout={openLogoutConfirm}
                  onRefresh={() => refreshDashboard().catch(() => undefined)}
                />
              )}
              {uiState.activePage === 'settings' && <SettingsPanel snapshot={appSnapshot} busy={busy} onUpdatePreferences={savePreferences} />}
            </section>
          </main>

          {dialogState.type === 'account' && (
            <AccountDialog
              open
              mode={dialogState.mode}
              account={dialogState.mode === 'edit' ? findAccount(appSnapshot, dialogState.accountId) : null}
              busy={busy}
              onClose={closeDialog}
              onSubmit={submitAccount}
            />
          )}

          {dialogState.type === 'confirmDelete' && (
            <ConfirmDialog
              open
              title="删除账号"
              message="确定删除这个账号吗？删掉后需要重新添加。"
              confirmText="删除"
              danger
              busy={busy}
              onClose={closeDialog}
              onConfirm={() => confirmDelete(dialogState.accountId)}
            />
          )}

          {dialogState.type === 'confirmLogout' && (
            <ConfirmDialog
              open
              title="下线本机设备"
              message="下线后本机会断网，需要重新登录认证。"
              confirmText="确认下线"
              busy={busy}
              onClose={closeDialog}
              onConfirm={confirmLogout}
            />
          )}
        </div>
      )}
    </FluentProvider>
  );
}

type NavButtonProps = {
  active: boolean;
  icon: ReactElement;
  label: string;
  page: UiState['activePage'];
};

function NavButton({ active, icon, label, page }: NavButtonProps) {
  const styles = useShellStyles();

  return (
    <Button
      appearance="subtle"
      className={styles.navButton}
      style={
        active
          ? {
              backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground1) 74%, transparent)',
              borderLeft: '3px solid var(--colorBrandForeground1)'
            }
          : undefined
      }
      icon={icon}
      onClick={() => setActivePage(page)}
    >
      {label}
    </Button>
  );
}

type StartupViewProps = {
  loadingMessage: string;
  error: string;
  detail: string;
};

function StartupView({ loadingMessage, error, detail }: StartupViewProps) {
  const styles = useShellStyles();

  return (
    <div className={styles.frame} style={{ display: 'grid', gridTemplateColumns: '1fr', placeItems: 'center' }}>
      <Card appearance="filled" className={styles.compactCard} style={{ width: 'min(100%, 360px)', textAlign: 'center' }}>
        <Wifi1Regular style={{ fontSize: 32 }} />
        <Text as="h1" size={600} weight="semibold">
          MUC-student
        </Text>
        <div className={styles.row} style={{ justifyContent: 'center' }}>
          <Spinner size="tiny" />
          <Text className={styles.muted}>{loadingMessage || '启动中...'}</Text>
        </div>
        {error && (
          <div className={styles.stack} style={{ textAlign: 'left', marginTop: 12 }}>
            <Text weight="semibold" className={styles.danger}>
              {error}
            </Text>
            {detail && <Text className={styles.muted}>{detail}</Text>}
          </div>
        )}
      </Card>
    </div>
  );
}
