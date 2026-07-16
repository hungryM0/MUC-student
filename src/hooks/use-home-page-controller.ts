import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  type AccountDto,
  type AppSnapshotDto,
  addAccount,
  bootstrapApp,
  deleteAccount,
  exportAccountPool,
  importAccountPool,
  loginSelectedAccount,
  logoutLocalDevice,
  readErrorMessage,
  refreshDashboard,
  selectAccount,
  updateAccount,
  updatePreferences,
} from "@/lib/muc";
import type {
  AccountFormState,
  AccountPoolDialogMode,
  Preferences,
  RunningAction,
} from "@/components/home/types";

export function useHomePageController() {
  const [snapshot, setSnapshot] = useState<AppSnapshotDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [errorText, setErrorText] = useState("");
  const [selectingId, setSelectingId] = useState("");
  const [loginAccountId, setLoginAccountId] = useState("");
  const [runningAction, setRunningAction] = useState<RunningAction | null>(
    null,
  );
  const [savingAccount, setSavingAccount] = useState(false);
  const [deletingAccountId, setDeletingAccountId] = useState("");
  const [accountToDelete, setAccountToDelete] = useState<AccountDto | null>(
    null,
  );
  const [accountForm, setAccountForm] = useState<AccountFormState | null>(null);
  const [accountPoolMode, setAccountPoolMode] =
    useState<AccountPoolDialogMode | null>(null);
  const [accountPoolCode, setAccountPoolCode] = useState("");
  const [accountPoolPassphrase, setAccountPoolPassphrase] = useState("");
  const [accountPoolBusy, setAccountPoolBusy] = useState(false);
  const [accountPoolResult, setAccountPoolResult] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [preferencesErrorText, setPreferencesErrorText] = useState("");
  const [preferencesSaving, setPreferencesSaving] = useState(false);

  useEffect(() => {
    const initTrayMenu = async () => {
      try {
        await invoke("update_tray_menu", {
          showText: "显示窗口",
          quitText: "退出",
        });
      } catch {
        setErrorText("托盘菜单初始化失败");
      }
    };

    void initTrayMenu();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;

    void listen<AppSnapshotDto>("muc://state-updated", (event) => {
      if (!disposed) {
        setSnapshot(event.payload);
      }
    }).then((handler) => {
      if (disposed) {
        handler();
      } else {
        unlisten = handler;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    void loadSnapshot();
  }, []);

  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();
    const unlisten = appWindow.onCloseRequested(async (event) => {
      if (snapshot?.preferences.minimizeToTrayOnClose) {
        event.preventDefault();
        await appWindow.hide();
      }
    });

    return () => {
      void unlisten.then((handler) => handler());
    };
  }, [snapshot?.preferences.minimizeToTrayOnClose]);

  const isBusy =
    loading ||
    !!selectingId ||
    !!runningAction ||
    savingAccount ||
    !!deletingAccountId ||
    accountPoolBusy ||
    !!snapshot?.loginState.running ||
    !!snapshot?.refreshState.running;
  const canLogoutLocalDevice = !!snapshot?.accounts.some(
    (account) => account.canLogoutLocalDevice,
  );

  async function loadSnapshot() {
    setLoading(true);
    setErrorText("");
    try {
      setSnapshot(await bootstrapApp());
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setLoading(false);
    }
  }

  async function runSnapshotAction(
    action: RunningAction,
    task: () => Promise<AppSnapshotDto>,
  ) {
    setRunningAction(action);
    setErrorText("");
    try {
      setSnapshot(await task());
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setRunningAction(null);
    }
  }

  async function handleLoginAccount(account: AccountDto) {
    if (isBusy) {
      return;
    }

    setLoginAccountId(account.id);
    setRunningAction("login");
    setErrorText("");

    try {
      if (account.id !== snapshot?.selectedAccountId) {
        setSelectingId(account.id);
        setSnapshot(await selectAccount(account.id));
        setSelectingId("");
      }
      setSnapshot(await loginSelectedAccount());
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setSelectingId("");
      setLoginAccountId("");
      setRunningAction(null);
    }
  }

  function openAddAccountForm() {
    setErrorText("");
    setAccountForm({
      accountId: "",
      remarkName: "",
      username: "",
      password: "",
    });
  }

  function openEditAccountForm(account: AccountDto) {
    setErrorText("");
    setAccountForm({
      accountId: account.id,
      remarkName: account.remarkName,
      username: account.username,
      password: "",
    });
  }

  async function handleSaveAccount() {
    if (!accountForm || savingAccount) {
      return;
    }

    setSavingAccount(true);
    setErrorText("");

    try {
      const nextSnapshot = accountForm.accountId
        ? await updateAccount(
            accountForm.accountId,
            accountForm.remarkName,
            accountForm.username,
            accountForm.password,
          )
        : await addAccount(
            accountForm.remarkName,
            accountForm.username,
            accountForm.password,
          );
      setSnapshot(nextSnapshot);
      setAccountForm(null);
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setSavingAccount(false);
    }
  }

  function requestDeleteAccount(account: AccountDto) {
    if (isBusy || deletingAccountId) {
      return;
    }

    setAccountToDelete(account);
  }

  function closeDeleteAccountDialog() {
    if (deletingAccountId) {
      return;
    }

    setAccountToDelete(null);
  }

  async function confirmDeleteAccount() {
    if (!accountToDelete || deletingAccountId) {
      return;
    }

    const account = accountToDelete;
    setAccountToDelete(null);
    setDeletingAccountId(account.id);
    setErrorText("");

    await new Promise((resolve) => window.setTimeout(resolve, 300));

    try {
      setSnapshot(await deleteAccount(account.id));
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setDeletingAccountId("");
    }
  }

  function openAccountPoolDialog(mode: AccountPoolDialogMode) {
    setErrorText("");
    setAccountPoolMode(mode);
    setAccountPoolCode("");
    setAccountPoolPassphrase("");
    setAccountPoolResult("");
  }

  function closeAccountPoolDialog() {
    if (accountPoolBusy) {
      return;
    }

    setAccountPoolMode(null);
  }

  async function handleExportAccountPool() {
    if (accountPoolBusy) {
      return;
    }

    setAccountPoolBusy(true);
    setErrorText("");
    setAccountPoolResult("");

    try {
      setAccountPoolCode(await exportAccountPool(accountPoolPassphrase));
      setAccountPoolResult("已生成号池码");
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setAccountPoolBusy(false);
    }
  }

  async function handleImportAccountPool() {
    if (accountPoolBusy) {
      return;
    }

    setAccountPoolBusy(true);
    setErrorText("");
    setAccountPoolResult("");

    try {
      const result = await importAccountPool(
        accountPoolCode,
        accountPoolPassphrase,
      );
      setSnapshot(result.snapshot);
      setAccountPoolResult(
        `导入 ${result.importedCount} 个，覆盖 ${result.overwrittenCount} 个`,
      );
      setAccountPoolMode(null);
    } catch (error) {
      setErrorText(readErrorMessage(error));
    } finally {
      setAccountPoolBusy(false);
    }
  }

  async function copyAccountPoolCode() {
    if (!accountPoolCode.trim()) {
      return;
    }

    try {
      await navigator.clipboard.writeText(accountPoolCode);
      setAccountPoolResult("已复制");
    } catch {
      setErrorText("复制号池码失败");
    }
  }

  async function togglePreference(key: keyof Preferences) {
    const preferences = snapshot?.preferences;
    if (!preferences || preferencesSaving) {
      return;
    }

    setPreferencesSaving(true);
    setPreferencesErrorText("");

    try {
      const nextSnapshot = await updatePreferences({
        ...preferences,
        [key]: !preferences[key],
      });
      setSnapshot(nextSnapshot);
      setErrorText("");
    } catch (error) {
      setPreferencesErrorText(readErrorMessage(error));
    } finally {
      setPreferencesSaving(false);
    }
  }

  return {
    snapshot,
    loading,
    errorText,
    clearError: () => {
      setErrorText("");
      setPreferencesErrorText("");
    },
    selectingId,
    loginAccountId,
    runningAction,
    savingAccount,
    deletingAccountId,
    accountToDelete,
    accountForm,
    accountPoolMode,
    accountPoolCode,
    accountPoolPassphrase,
    accountPoolBusy,
    accountPoolResult,
    settingsOpen,
    aboutOpen,
    preferencesErrorText,
    preferencesSaving,
    isBusy,
    canLogoutLocalDevice,
    setAccountForm,
    setAccountPoolCode,
    setAccountPoolPassphrase,
    handleLoginAccount,
    openAddAccountForm,
    openEditAccountForm,
    handleSaveAccount,
    requestDeleteAccount,
    closeDeleteAccountDialog,
    confirmDeleteAccount,
    openAccountPoolDialog,
    closeAccountPoolDialog,
    handleExportAccountPool,
    handleImportAccountPool,
    copyAccountPoolCode,
    refreshDashboard: () => runSnapshotAction("refresh", refreshDashboard),
    logoutLocalDevice: () => runSnapshotAction("logout", logoutLocalDevice),
    openSettings: () => {
      setPreferencesErrorText("");
      setSettingsOpen(true);
    },
    closeSettings: () => setSettingsOpen(false),
    openAbout: () => setAboutOpen(true),
    closeAbout: () => setAboutOpen(false),
    togglePreference,
  };
}
