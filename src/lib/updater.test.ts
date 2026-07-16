import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import type { Update } from "@tauri-apps/plugin-updater";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fetchAndroidUpdateFeed } from "@/lib/muc";
import { isAndroid } from "@/lib/utils";
import { checkForUpdates, downloadAndInstall } from "./updater";

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

vi.mock("@/lib/utils", () => ({
  isAndroid: vi.fn(),
}));

vi.mock("@/lib/muc", () => ({
  fetchAndroidUpdateFeed: vi.fn(),
}));

const checkMock = vi.mocked(check);
const getVersionMock = vi.mocked(getVersion);
const relaunchMock = vi.mocked(relaunch);
const openUrlMock = vi.mocked(openUrl);
const isAndroidMock = vi.mocked(isAndroid);
const fetchAndroidUpdateFeedMock = vi.mocked(fetchAndroidUpdateFeed);

describe("updater", () => {
  beforeEach(() => {
    checkMock.mockReset();
    getVersionMock.mockReset();
    relaunchMock.mockReset();
    openUrlMock.mockReset();
    isAndroidMock.mockReset();
    fetchAndroidUpdateFeedMock.mockReset();
    isAndroidMock.mockReturnValue(false);
  });

  it("uses Tauri updater on desktop", async () => {
    const update = { version: "2.0.1" } as Update;
    checkMock.mockResolvedValue(update);

    await expect(checkForUpdates()).resolves.toEqual({
      status: "available",
      update,
    });
    expect(fetchAndroidUpdateFeedMock).not.toHaveBeenCalled();
  });

  it("returns error status when desktop update check fails", async () => {
    const error = new Error("updater unavailable");
    checkMock.mockRejectedValue(error);

    await expect(checkForUpdates()).resolves.toEqual({
      status: "error",
      error,
    });
  });

  it("uses Android feed and ignores stale versions", async () => {
    isAndroidMock.mockReturnValue(true);
    getVersionMock.mockResolvedValue("2.0.0");
    fetchAndroidUpdateFeedMock.mockResolvedValue({
      version: "2.0.0",
      notes: "desktop notes",
      android: {
        version: "2.0.1",
        url: "https://example.test/MUC-student.apk",
        notes: "android notes",
      },
    });

    await expect(checkForUpdates()).resolves.toEqual({
      status: "android-available",
      update: {
        version: "2.0.1",
        url: "https://example.test/MUC-student.apk",
        notes: "android notes",
      },
    });

    fetchAndroidUpdateFeedMock.mockResolvedValue({
      android: {
        version: "v2.0.0",
        url: "https://example.test/MUC-student.apk",
      },
    });

    await expect(checkForUpdates()).resolves.toEqual({
      status: "up-to-date",
    });
  });

  it("opens Android update URL instead of desktop installer", async () => {
    isAndroidMock.mockReturnValue(true);

    await expect(
      downloadAndInstall({
        version: "2.0.1",
        url: "https://example.test/MUC-student.apk",
      }),
    ).resolves.toBe(true);

    expect(openUrlMock).toHaveBeenCalledWith(
      "https://example.test/MUC-student.apk",
    );
    expect(checkMock).not.toHaveBeenCalled();
    expect(relaunchMock).not.toHaveBeenCalled();
  });

  it("downloads desktop update, reports accumulated progress, and relaunches", async () => {
    const onProgress = vi.fn();
    const performanceNowMock = vi
      .spyOn(performance, "now")
      .mockReturnValueOnce(1000)
      .mockReturnValueOnce(2000)
      .mockReturnValueOnce(3000)
      .mockReturnValueOnce(4000);
    const update = {
      downloadAndInstall: vi.fn(async (callback) => {
        callback({ event: "Started", data: { contentLength: 10 } });
        callback({ event: "Progress", data: { chunkLength: 4 } });
        callback({ event: "Progress", data: { chunkLength: 6 } });
        callback({ event: "Finished", data: {} });
      }),
    } as unknown as Update;

    await expect(downloadAndInstall(update, onProgress)).resolves.toBe(true);

    expect(update.downloadAndInstall).toHaveBeenCalledOnce();
    expect(onProgress).toHaveBeenNthCalledWith(1, {
      event: "Started",
      data: { contentLength: 10, downloaded: 0 },
    });
    expect(onProgress).toHaveBeenNthCalledWith(2, {
      event: "Progress",
      data: {
        chunkLength: 4,
        contentLength: 10,
        downloaded: 4,
        speedBytesPerSecond: 4,
      },
    });
    expect(onProgress).toHaveBeenNthCalledWith(3, {
      event: "Progress",
      data: {
        chunkLength: 6,
        contentLength: 10,
        downloaded: 10,
        speedBytesPerSecond: 5,
      },
    });
    expect(onProgress).toHaveBeenNthCalledWith(4, {
      event: "Finished",
      data: {
        contentLength: 10,
        downloaded: 10,
        speedBytesPerSecond: 10 / 3,
      },
    });
    expect(relaunchMock).toHaveBeenCalledOnce();
    performanceNowMock.mockRestore();
  });
});
