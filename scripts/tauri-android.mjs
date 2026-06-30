import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");

const command = process.argv[2];
const passthroughArgs = process.argv.slice(3);
if (!command || !["dev", "check", "build"].includes(command)) {
  console.error("用法：node scripts/tauri-android.mjs <dev|check|build>");
  process.exit(1);
}

const env = { ...process.env };
const androidSdk = findAndroidSdk();
const ndkHome = findAndroidNdk(androidSdk);
const javaHome = findJavaHome();
const ndkBin = path.join(
  ndkHome,
  "toolchains",
  "llvm",
  "prebuilt",
  "windows-x86_64",
  "bin",
);
const originalPath = env.PATH ?? env.Path ?? "";
const androidApiLevel = 34;
const adbExecutable = path.join(androidSdk, "platform-tools", "adb.exe");

env.ANDROID_HOME = androidSdk;
env.ANDROID_SDK_ROOT = androidSdk;
env.ANDROID_NDK_HOME = ndkHome;
env.NDK_HOME = ndkHome;
env.JAVA_HOME = javaHome;
env.PATH = [path.join(javaHome, "bin"), ndkBin, originalPath]
  .filter(Boolean)
  .join(path.delimiter);
env.Path = env.PATH;

env.ORG_GRADLE_PROJECT_abiList = "arm64-v8a";
env.ORG_GRADLE_PROJECT_archList = "arm64";
env.ORG_GRADLE_PROJECT_targetList = "aarch64";

for (const [target, triple] of [
  ["aarch64_linux_android", "aarch64-linux-android"],
]) {
  env[`CC_${target}`] = path.join(
    ndkBin,
    `${triple}${androidApiLevel}-clang.cmd`,
  );
  env[`CXX_${target}`] = path.join(
    ndkBin,
    `${triple}${androidApiLevel}-clang++.cmd`,
  );
}

const result =
  command === "check"
    ? spawnSync("cargo", ["check", "--target", "aarch64-linux-android"], {
        cwd: rootDir,
        env,
        stdio: "inherit",
        shell: process.platform === "win32",
      })
    : spawnSync(
        "pnpm",
        ["tauri", "android", command, ...tauriArgs(command), ...passthroughArgs],
        {
        cwd: rootDir,
        env,
        stdio: "inherit",
        shell: process.platform === "win32",
      },
      );

process.exit(result.status ?? 1);

function findAndroidSdk() {
  const candidates = [
    process.env.ANDROID_HOME,
    process.env.ANDROID_SDK_ROOT,
    path.join(os.homedir(), "AppData", "Local", "Android", "Sdk"),
  ].filter(Boolean);

  const sdk = candidates.find((item) =>
    existsSync(path.join(item, "platform-tools", "adb.exe")),
  );
  if (!sdk) {
    throw new Error(
      "找不到 Android SDK。请安装 Android SDK，或设置 ANDROID_HOME。",
    );
  }
  return sdk;
}

function tauriArgs(command) {
  if (command === "build") {
    return ["--target", "aarch64"];
  }

  if (command !== "dev") {
    return [];
  }

  if (passthroughArgs.includes("--host") || passthroughArgs.includes("--open")) {
    return [];
  }

  setupAdbReverse();
  return ["--host", "127.0.0.1"];
}

function findAndroidNdk(androidSdk) {
  const ndkRoot = path.join(androidSdk, "ndk");
  if (!existsSync(ndkRoot)) {
    throw new Error("找不到 Android NDK。请安装 NDK。");
  }

  const candidates = spawnSync(
    "powershell",
    [
      "-NoProfile",
      "-Command",
      `Get-ChildItem -LiteralPath '${ndkRoot.replaceAll("'", "''")}' -Directory | Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName`,
    ],
    { encoding: "utf8" },
  )
    .stdout.trim()
    .split(/\r?\n/)
    .filter(Boolean);

  const ndk = candidates.find((item) =>
    existsSync(
      path.join(
        item,
        "toolchains",
        "llvm",
        "prebuilt",
        "windows-x86_64",
        "bin",
      ),
    ),
  );
  if (!ndk) {
    throw new Error("找不到可用 Android NDK toolchain。");
  }
  return ndk;
}

function findJavaHome() {
  const candidates = [
    process.env.JAVA_HOME,
    path.join("C:", "Program Files", "Android", "Android Studio", "jbr"),
    path.join("C:", "Program Files", "Java", "jdk-17.0.2"),
    path.join("C:", "Program Files", "Java", "jdk-21"),
    path.join("C:", "Program Files", "Java", "jdk-25"),
  ].filter(Boolean);

  const javaHome = candidates.find((item) =>
    existsSync(path.join(item, "bin", "java.exe")),
  );
  if (!javaHome) {
    throw new Error("找不到 Java。请安装 JDK 17+，或设置 JAVA_HOME。");
  }
  return javaHome;
}

function setupAdbReverse() {
  if (!existsSync(adbExecutable)) {
    console.warn("未找到 adb，跳过端口反向代理。");
    return;
  }

  const devices = listAdbDevices();
  if (devices.length === 0) {
    console.warn("未发现已连接 Android 设备，跳过端口反向代理。");
    return;
  }

  for (const serial of devices) {
    reversePort(serial, 1420);
    reversePort(serial, 1421);
  }
}

function listAdbDevices() {
  const result = spawnSync(adbExecutable, ["devices"], { encoding: "utf8" });
  if (result.status !== 0) {
    console.warn("读取 adb 设备列表失败，跳过端口反向代理。");
    return [];
  }

  return result.stdout
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("List of devices attached"))
    .map((line) => line.split(/\s+/))
    .filter((parts) => parts[1] === "device")
    .map((parts) => parts[0]);
}

function reversePort(serial, port) {
  const result = spawnSync(
    adbExecutable,
    ["-s", serial, "reverse", `tcp:${port}`, `tcp:${port}`],
    { encoding: "utf8" },
  );

  if (result.status !== 0) {
    const reason = [result.stdout, result.stderr].join("\n").trim();
    console.warn(
      `设备 ${serial} 的 adb reverse tcp:${port} 失败${reason ? `：${reason}` : ""}`,
    );
  }
}
