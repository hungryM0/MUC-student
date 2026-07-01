import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  addAccount,
  exportAccountPool,
  importAccountPool,
  readErrorMessage,
  refreshDashboard,
  updateAccount,
  updatePreferences,
} from "./muc";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("muc invoke bridge", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({} as never);
  });

  it("passes account create and update payloads to Tauri commands", async () => {
    await addAccount("主号", "20260001", "secret");
    await updateAccount("account-1", "主号", "20260001", "   ");
    await updateAccount("account-1", "主号", "20260001", "new-secret");

    expect(invokeMock).toHaveBeenNthCalledWith(1, "add_account", {
      remarkName: "主号",
      username: "20260001",
      password: "secret",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "update_account", {
      accountId: "account-1",
      remarkName: "主号",
      username: "20260001",
      password: null,
    });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "update_account", {
      accountId: "account-1",
      remarkName: "主号",
      username: "20260001",
      password: "new-secret",
    });
  });

  it("passes preference fields explicitly", async () => {
    await updatePreferences({
      minimizeToTrayOnClose: true,
      launchOnStartup: false,
      autoSwitchAccountOnTrafficExhausted: true,
    });

    expect(invokeMock).toHaveBeenCalledWith("update_preferences", {
      minimizeToTrayOnClose: true,
      launchOnStartup: false,
      autoSwitchAccountOnTrafficExhausted: true,
    });
  });

  it("keeps command names stable for refresh", async () => {
    await refreshDashboard();

    expect(invokeMock).toHaveBeenCalledWith("refresh_dashboard");
  });

  it("passes account pool transfer payloads to Tauri commands", async () => {
    await exportAccountPool("share-pass");
    await importAccountPool("MUCPOOL1.payload", "share-pass");

    expect(invokeMock).toHaveBeenNthCalledWith(1, "export_account_pool", {
      passphrase: "share-pass",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "import_account_pool", {
      code: "MUCPOOL1.payload",
      passphrase: "share-pass",
    });
  });

  it("reads structured command errors before falling back to string conversion", () => {
    expect(
      readErrorMessage({ message: "账号不能为空", code: "VALIDATION" }),
    ).toBe("账号不能为空");
    expect(readErrorMessage("boom")).toBe("boom");
  });
});
