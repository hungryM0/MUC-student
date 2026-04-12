# -*- mode: python ; coding: utf-8 -*-

import sys
from pathlib import Path

project_root = Path(SPECPATH).resolve()
site_packages = Path(sys.base_prefix) / "Lib" / "site-packages"
ddddocr_old_model = site_packages / "ddddocr" / "common_old.onnx"

# Whitelist: only modules that need explicit hidden imports.
WHITELIST_PACKAGES = [
    "qfluentwidgets",
    "ddddocr",
]

# Whitelist: critical Qt modules used by the app entry and pages.
WHITELIST_HIDDEN_IMPORTS = [
    "PySide6.QtCore",
    "PySide6.QtGui",
    "PySide6.QtWidgets",
]

# Blacklist: explicitly exclude obvious heavy/unused stacks.
BLACKLIST_MODULES = [
    "pytest",
    "unittest",
    "tkinter",
    "matplotlib",
    "pandas",
    "scipy",
    "IPython",
    "jupyter",
    "notebook",
    "pyinstaller",
    "PySide6.QtPdf",
    "PySide6.QtQml",
    "PySide6.QtQmlMeta",
    "PySide6.QtQmlModels",
    "PySide6.QtQmlWorkerScript",
    "PySide6.QtQuick",
    "PySide6.QtVirtualKeyboard",
]

# Binary blacklist: OpenCV video I/O codec DLL is not required for captcha OCR.
BINARY_BLACKLIST_PATTERNS = [
    "opencv_videoio_ffmpeg",
    "pyside6\\qt6pdf",
    "pyside6\\qt6qml",
    "pyside6\\qt6quick",
    "pyside6\\qt6virtualkeyboard",
    "pyside6\\translations\\",
    "pyside6\\plugins\\platforminputcontexts\\qtvirtualkeyboardplugin",
    "pyside6\\plugins\\networkinformation\\",
    "pyside6\\plugins\\platforms\\qdirect2d",
    "pyside6\\plugins\\platforms\\qminimal",
    "pyside6\\plugins\\platforms\\qoffscreen",
]

datas = [
    (str(project_root / "accounts.json"), "."),
    (str(project_root / "app_state.json"), "."),
]

if ddddocr_old_model.exists():
    datas.append((str(ddddocr_old_model), "ddddocr"))

binaries = []
hiddenimports = list(WHITELIST_HIDDEN_IMPORTS)

hiddenimports.extend(WHITELIST_PACKAGES)

hiddenimports = sorted(set(hiddenimports))

a = Analysis(
    ["main.py"],
    pathex=[str(project_root)],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=BLACKLIST_MODULES,
    noarchive=False,
    optimize=0,
)

def _keep_binary(entry: tuple[str, ...]) -> bool:
    normalized = " ".join(str(part).lower() for part in entry)
    return not any(pattern in normalized for pattern in BINARY_BLACKLIST_PATTERNS)

a.binaries = [entry for entry in a.binaries if _keep_binary(entry)]

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name="MUC-student",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)