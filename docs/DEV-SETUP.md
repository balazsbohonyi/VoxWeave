# VoxFlow — Dev Environment Setup

> Setup guide for Tauri v2 + Rust + Vue 3 + TypeScript development.

- [Windows](#windows)
- [macOS](#macos)

---

## Windows

### 1. VS Code

Download and install VS Code from https://code.visualstudio.com/

#### Extensions

Install all of the following from the VS Code Marketplace:

**Rust & Tauri**

| Extension | Link | Purpose |
|---|---|---|
| **rust-analyzer** | [marketplace](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) | Rust LSP — completions, type hints, inline errors. Non-negotiable. |
| **Even Better TOML** | [marketplace](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) | `Cargo.toml` syntax highlighting + validation |
| **CodeLLDB** | [marketplace](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) | Rust debugger — attach to the Tauri backend process |
| **Tauri** | [marketplace](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) | `tauri.conf.json` schema + Tauri command snippets |

**Vue & TypeScript**

| Extension | Link | Purpose |
|---|---|---|
| **Vue - Official** (Volar) | [marketplace](https://marketplace.visualstudio.com/items?itemName=Vue.volar) | Vue 3 + TypeScript support. Replaces the old Vetur. |

**Code Quality**

| Extension | Link | Purpose |
|---|---|---|
| **ESLint** | [marketplace](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint) | Linting for TypeScript + Vue |
| **Prettier** | [marketplace](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) | Code formatting |

**Optional but Recommended**

| Extension | Link | Purpose |
|---|---|---|
| **Error Lens** | [marketplace](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens) | Shows Rust/TS errors inline next to the code — pairs great with rust-analyzer |
| **GitLens** | [marketplace](https://marketplace.visualstudio.com/items?itemName=eamodio.gitlens) | Enhanced Git history and blame |

---

### 2. Microsoft C++ Build Tools

Required by Rust's MSVC toolchain and by whisper.cpp native compilation.

1. Download from https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Run the installer
3. Select the **"Desktop development with C++"** workload — this pulls in the MSVC compiler and Windows SDK
4. Complete installation and **restart your machine**

> If you already have Visual Studio 2019 or 2022 installed with the C++ workload, you can skip this step.

---

### 3. Rust

Install Rust via `rustup` — the official Rust toolchain manager.

1. Download the installer from https://rustup.rs/
2. Run it and follow the prompts
3. When asked about the toolchain, make sure **MSVC** is selected as the host triple (e.g. `x86_64-pc-windows-msvc`) — **not** GNU
4. After installation, restart your terminal and verify:

```powershell
rustc --version
cargo --version
```

Keep Rust up to date:

```powershell
rustup update
```

---

### 4. Node.js

Requires Node.js 20 or later. Install via `winget` (built into Windows 10/11):

```powershell
winget install OpenJS.NodeJS.LTS
```

Or download the installer directly from https://nodejs.org/

Verify:

```powershell
node --version
```

---

### 5. pnpm

```powershell
npm install -g pnpm@latest-10
```

Verify:

```powershell
pnpm --version
```

---

### 6. CMake *(local transcription only)*

CMake is required only when building with the `local-transcription` cargo feature (whisper.cpp/whisper-rs). The base app builds and runs fine without it.

Install via winget:

```powershell
winget install Kitware.CMake
```

Or download from https://cmake.org/download/ — use the Windows x64 installer and select **"Add CMake to the system PATH"** during installation.

Verify after restarting your terminal:

```powershell
cmake --version
```

---

### 7. LLVM / libclang *(local transcription only)*

Also required for the `local-transcription` feature. `whisper-rs-sys` uses `bindgen` to generate Rust bindings for whisper.cpp's C API, and `bindgen` needs `libclang.dll` at build time.

Install via winget:

```powershell
winget install LLVM.LLVM
```

Or download the `LLVM-xx.x.x-win64.exe` installer from https://github.com/llvm/llvm-project/releases — select **"Add LLVM to the system PATH"** during installation.

Verify after restarting your terminal:

```powershell
clang --version
```

If the build still can't find `libclang.dll`, set the env var manually in your terminal session:

```powershell
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

---

### 8. WebView2 Runtime

Tauri uses Microsoft Edge WebView2 to render the frontend.

**On Windows 10 (version 1803+) and Windows 11 it is pre-installed** — you likely don't need to do anything.

If for some reason it's missing, download the Evergreen Bootstrapper from:
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

---

### 9. WiX Toolset *(MSI builds only)*

Required only when building `.msi` installer packages. Not needed for development or NSIS builds.

```powershell
winget install WiXToolset.WiXToolset
```

> Skip this during development. NSIS (`.exe` installer) builds work without WiX.

---

### 10. Git

```powershell
winget install Git.Git
```

Verify:

```powershell
git --version
```

---

### Running the dev server

**Standard build** (no CMake/LLVM required):

```powershell
pnpm tauri dev
```

**With local transcription** (requires CMake + LLVM from steps 6–7):

```powershell
pnpm tauri dev --features local-transcription
```

The first run compiles all Rust dependencies — this takes several minutes. Subsequent runs are much faster.

---

### Windows Quick Reference Checklist

```
[ ] VS Code installed
[ ] rust-analyzer extension
[ ] Even Better TOML extension
[ ] CodeLLDB extension
[ ] Tauri extension
[ ] Vue - Official (Volar) extension
[ ] ESLint extension
[ ] Prettier extension
[ ] Microsoft C++ Build Tools (Desktop development with C++)
[ ] Rust via rustup (MSVC toolchain, stable channel)
[ ] Node.js 20+
[ ] pnpm
[ ] CMake (only for --features local-transcription)
[ ] LLVM (only for --features local-transcription)
[ ] WebView2 Runtime (pre-installed on Win10 1803+ / Win11)
[ ] WiX Toolset (only if building .msi)
[ ] Git
```

---

## macOS

> macOS support is not yet implemented — the platform abstraction layer (audio capture, text injection, foreground window detection) is Windows-only at this stage. The sections below describe what will be needed once macOS stubs are wired up.

### 1. Xcode Command Line Tools

```bash
xcode-select --install
```

This installs the C/C++ compiler, `git`, and other build essentials.

---

### 2. Homebrew

If not already installed:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

---

### 3. Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify:

```bash
rustc --version
cargo --version
```

---

### 4. Node.js

```bash
brew install node@20
```

---

### 5. pnpm

```bash
npm install -g pnpm@latest-10
```

---

### 6. CMake *(local transcription only)*

```bash
brew install cmake
```

---

### 7. LLVM / libclang *(local transcription only)*

macOS ships with Apple Clang which does not include `libclang` in the form bindgen expects. Install the full LLVM toolchain via Homebrew:

```bash
brew install llvm
```

Then export the path so bindgen can find it (add to your shell profile):

```bash
export LIBCLANG_PATH="$(brew --prefix llvm)/lib"
```

---

### 8. Running the dev server

```bash
pnpm tauri dev
# or with local transcription:
pnpm tauri dev --features local-transcription
```

---

### macOS Quick Reference Checklist

```
[ ] Xcode Command Line Tools
[ ] Homebrew
[ ] Rust via rustup
[ ] Node.js 20+
[ ] pnpm
[ ] CMake (only for --features local-transcription)
[ ] LLVM via Homebrew (only for --features local-transcription)
```
