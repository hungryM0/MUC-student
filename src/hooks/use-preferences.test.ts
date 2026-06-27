import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { usePreferences } from "./use-preferences";
import type { AppSnapshotDto } from "@/lib/muc";

const mucMock = vi.hoisted(() => ({
  getAppSnapshot: vi.fn(),
  updatePreferences: vi.fn(),
  readErrorMessage: vi.fn((error: unknown) =>
    typeof error === "object" && error && "message" in error
      ? String((error as { message: string }).message)
      : String(error),
  ),
}));

vi.mock("@/lib/muc", () => mucMock);

const basePreferences: AppSnapshotDto["preferences"] = {
  minimizeToTrayOnClose: true,
  launchOnStartup: false,
  autoSwitchAccountOnTrafficExhausted: false,
};

function snapshotWithPreferences(
  preferences: AppSnapshotDto["preferences"],
): AppSnapshotDto {
  return {
    network: {
      isOnline: true,
      statusText: "IP 已识别",
      ip: "10.151.119.57",
      checkedAt: "2026-01-01T00:00:00+08:00",
    },
    accounts: [],
    selectedAccountId: "",
    currentOnlineAccountId: "",
    poolQuota: {
      usedTrafficText: "-",
      productBalanceText: "-",
      includedPackageText: "",
      progressPercent: null,
    },
    loginState: {
      running: false,
      lastLoginTime: null,
      resultText: "未执行",
      message: "-",
    },
    refreshState: {
      running: false,
      lastQuotaRefreshTime: null,
    },
    preferences,
  };
}

describe("usePreferences", () => {
  beforeEach(() => {
    mucMock.getAppSnapshot.mockReset();
    mucMock.updatePreferences.mockReset();
    mucMock.readErrorMessage.mockClear();
  });

  it("loads preferences only when active", async () => {
    mucMock.getAppSnapshot.mockResolvedValue(
      snapshotWithPreferences(basePreferences),
    );
    const { result, rerender } = renderHook(
      ({ active }) => usePreferences(active),
      { initialProps: { active: false } },
    );

    expect(result.current.preferences).toBeNull();
    expect(mucMock.getAppSnapshot).not.toHaveBeenCalled();

    rerender({ active: true });

    await waitFor(() => {
      expect(result.current.preferences).toEqual(basePreferences);
    });
    expect(result.current.errorText).toBe("");
  });

  it("optimistically toggles and accepts the saved snapshot", async () => {
    const savedPreferences = {
      ...basePreferences,
      launchOnStartup: true,
      minimizeToTrayOnClose: false,
    };
    mucMock.getAppSnapshot.mockResolvedValue(
      snapshotWithPreferences(basePreferences),
    );
    mucMock.updatePreferences.mockResolvedValue(
      snapshotWithPreferences(savedPreferences),
    );
    const { result } = renderHook(() => usePreferences(true));
    await waitFor(() => {
      expect(result.current.preferences).toEqual(basePreferences);
    });

    act(() => {
      result.current.togglePreference("launchOnStartup");
    });

    expect(result.current.preferences).toEqual({
      ...basePreferences,
      launchOnStartup: true,
    });
    expect(mucMock.updatePreferences).toHaveBeenCalledWith({
      ...basePreferences,
      launchOnStartup: true,
    });
    await waitFor(() => {
      expect(result.current.preferences).toEqual(savedPreferences);
    });
  });

  it("rolls back optimistic changes when saving fails", async () => {
    mucMock.getAppSnapshot.mockResolvedValue(
      snapshotWithPreferences(basePreferences),
    );
    mucMock.updatePreferences.mockRejectedValue(new Error("保存失败"));
    const { result } = renderHook(() => usePreferences(true));
    await waitFor(() => {
      expect(result.current.preferences).toEqual(basePreferences);
    });

    act(() => {
      result.current.togglePreference("autoSwitchAccountOnTrafficExhausted");
    });

    expect(result.current.preferences).toEqual({
      ...basePreferences,
      autoSwitchAccountOnTrafficExhausted: true,
    });
    await waitFor(() => {
      expect(result.current.preferences).toEqual(basePreferences);
      expect(result.current.errorText).toBe("保存失败");
    });
  });
});
