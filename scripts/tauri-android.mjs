import { existsSync, readdirSync } from "node:fs";
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
const ndkBin = findNdkBin(ndkHome);
const originalPath = env.PATH ?? env.Path ?? "";
const androidApiLevel = 34;

env.ANDROID_HOME = androidSdk;
env.ANDROID_SDK_ROOT = androidSdk;
env.ANDROID_NDK = ndkHome;
env.ANDROID_NDK_ROOT = ndkHome;
env.ANDROID_NDK_HOME = ndkHome;
env.NDK_HOME = ndkHome;
prependToPath(env, ndkBin);

env.ORG_GRADLE_PROJECT_abiList = "arm64-v8a";
env.ORG_GRADLE_PROJECT_archList = "arm64";
env.ORG_GRADLE_PROJECT_targetList = "aarch64";

for (const [target, triple] of [
  ["aarch64_linux_android", "aarch64-linux-android"],
]) {
  env[`CC_${target}`] = path.join(
    ndkBin,
    `${triple}${androidApiLevel}-clang${executableExtension()}`,
  );
  env[`CXX_${target}`] = path.join(
    ndkBin,
    `${triple}${androidApiLevel}-clang++${executableExtension()}`,
  );
}

if (command !== "check") {
  const javaHome = findJavaHome();
  env.JAVA_HOME = javaHome;
  prependToPath(env, path.join(javaHome, "bin"));
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
        [
          "tauri",
          "android",
          command,
          ...tauriArgs(command),
          ...passthroughArgs,
        ],
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
    ...defaultAndroidSdkCandidates(),
  ].filter(Boolean);

  const sdk = candidates.find((item) => existsSync(findAdbExecutable(item)));
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
  const ndkCandidates = [
    process.env.ANDROID_NDK_LATEST_HOME,
    process.env.ANDROID_NDK_HOME,
    process.env.ANDROID_NDK_ROOT,
    process.env.ANDROID_NDK,
    process.env.NDK_HOME,
    ...listChildDirectories(path.join(androidSdk, "ndk")),
  ].filter(Boolean);

  const ndk = ndkCandidates.find((item) => {
    try {
      return existsSync(findNdkBin(item));
    } catch {
      return false;
    }
  });
  if (!ndk) {
    throw new Error("找不到可用 Android NDK toolchain。");
  }
  return ndk;
}

function findJavaHome() {
  const javaFromPath = findJavaHomeFromPath();
  const candidates = [
    process.env.JAVA_HOME,
    javaFromPath,
    ...defaultJavaHomeCandidates(),
  ].filter(Boolean);

  const javaHome = candidates.find((item) =>
    existsSync(path.join(item, "bin", `java${executableExtension()}`)),
  );
  if (!javaHome) {
    throw new Error("找不到 Java。请安装 JDK 17+，或设置 JAVA_HOME。");
  }
  return javaHome;
}

function setupAdbReverse() {
  const adbExecutable = findAdbExecutable(androidSdk);
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
  const adbExecutable = findAdbExecutable(androidSdk);
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
  const adbExecutable = findAdbExecutable(androidSdk);
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

function executableExtension() {
  return process.platform === "win32" ? ".cmd" : "";
}

function findAdbExecutable(androidSdk) {
  return path.join(
    androidSdk,
    "platform-tools",
    process.platform === "win32" ? "adb.exe" : "adb",
  );
}

function findNdkBin(ndkHome) {
  const prebuiltRoot = path.join(ndkHome, "toolchains", "llvm", "prebuilt");
  const availableDirs = listChildDirectories(prebuiltRoot).map((item) =>
    path.basename(item),
  );

  for (const dirName of preferredNdkPrebuiltDirs()) {
    if (!availableDirs.includes(dirName)) {
      continue;
    }

    const binDir = path.join(prebuiltRoot, dirName, "bin");
    if (existsSync(binDir)) {
      return binDir;
    }
  }

  throw new Error("找不到当前平台可用的 Android NDK prebuilt toolchain。");
}

function preferredNdkPrebuiltDirs() {
  if (process.platform === "win32") {
    return ["windows-x86_64"];
  }

  if (process.platform === "darwin") {
    return process.arch === "arm64"
      ? ["darwin-arm64", "darwin-x86_64"]
      : ["darwin-x86_64", "darwin-arm64"];
  }

  return process.arch === "arm64"
    ? ["linux-aarch64", "linux-x86_64"]
    : ["linux-x86_64", "linux-aarch64"];
}

function defaultAndroidSdkCandidates() {
  if (process.platform === "win32") {
    return [path.join(os.homedir(), "AppData", "Local", "Android", "Sdk")];
  }

  if (process.platform === "darwin") {
    return [path.join(os.homedir(), "Library", "Android", "sdk")];
  }

  return [path.join(os.homedir(), "Android", "Sdk")];
}

function defaultJavaHomeCandidates() {
  if (process.platform === "win32") {
    return [
      path.join("C:", "Program Files", "Android", "Android Studio", "jbr"),
      path.join("C:", "Program Files", "Java", "jdk-17.0.2"),
      path.join("C:", "Program Files", "Java", "jdk-21"),
      path.join("C:", "Program Files", "Java", "jdk-25"),
    ];
  }

  if (process.platform === "darwin") {
    return [
      path.join(
        "/Applications",
        "Android Studio.app",
        "Contents",
        "jbr",
        "Contents",
        "Home",
      ),
      path.join(
        "/Library",
        "Java",
        "JavaVirtualMachines",
        "temurin-17.jdk",
        "Contents",
        "Home",
      ),
    ];
  }

  return [
    "/usr/lib/jvm/temurin-17-jdk-amd64",
    "/usr/lib/jvm/java-17-openjdk-amd64",
    "/usr/lib/jvm/java-17-openjdk-arm64",
  ];
}

function listChildDirectories(dirPath) {
  if (!existsSync(dirPath)) {
    return [];
  }

  return readdirSync(dirPath, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(dirPath, entry.name))
    .sort((left, right) =>
      path.basename(right).localeCompare(path.basename(left), undefined, {
        numeric: true,
        sensitivity: "base",
      }),
    );
}

function prependToPath(targetEnv, entry) {
  targetEnv.PATH = [entry, targetEnv.PATH ?? targetEnv.Path ?? ""]
    .filter(Boolean)
    .join(path.delimiter);
  targetEnv.Path = targetEnv.PATH;
}

function findJavaHomeFromPath() {
  const result = spawnSync("java", ["-XshowSettings:properties", "-version"], {
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) {
    return null;
  }

  const output = [result.stdout, result.stderr].join("\n");
  const line = output
    .split(/\r?\n/)
    .map((item) => item.trim())
    .find((item) => item.startsWith("java.home = "));

  if (!line) {
    return null;
  }

  const javaHome = line.slice("java.home = ".length).trim();
  return javaHome || null;
}
