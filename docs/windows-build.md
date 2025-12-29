# Building SSLBoard Desktop for Windows

This guide covers different approaches to build the Tauri app for Windows.

## Option 1: Build on Windows Machine (Easiest & Recommended)

This is the simplest and most reliable approach.

### Prerequisites on Windows

1. **Node.js** (v18+)
   - Download from [nodejs.org](https://nodejs.org/)

2. **Rust**
   - Install from [rustup.rs](https://rustup.rs/)
   - Run `rustup update stable`

3. **Microsoft Visual C++ Build Tools**
   - Download from [Microsoft](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Or install Visual Studio with the "Desktop development with C++" workload

4. **WebView2 Runtime** (usually pre-installed on Windows 10/11)
   - Tauri uses WebView2, which comes with Windows 10/11
   - For older systems, download from [Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/)

### Build Steps

1. Clone and navigate to the project:
   ```bash
   cd desktop
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Build for Windows:
   ```bash
   npm run tauri build
   ```

   The installer will be created in:
   - `src-tauri/target/release/bundle/msi/SSLBoard_<version>_x64_en-US.msi` (MSI installer)
   - `src-tauri/target/release/bundle/nsis/SSLBoard_<version>_x64-setup.exe` (NSIS installer, if configured)

## Option 2: GitHub Actions (Best for CI/CD) ✅ Already Configured

A GitHub Actions workflow is already set up at `.github/workflows/build.yml`. This approach builds Windows binaries automatically when you push a version tag.

### How It Works

The workflow automatically:
- **Builds and creates releases** when you push a tag starting with `v` (e.g., `v0.6.4`)
- **Attaches Windows installers** (MSI and NSIS) to the release automatically

### Creating a Release

1. **Update the version** in `package.json`:
   ```json
   "version": "0.6.4"
   ```

2. **Commit and push your changes**:
   ```bash
   git add package.json
   git commit -m "Bump version to 0.6.4"
   git push
   ```

3. **Create and push a tag**:
   ```bash
   git tag v0.6.4
   git push origin v0.6.4
   ```

   The workflow will automatically:
   - Build the Windows installer
   - Create a GitHub release with the tag name
   - Attach the MSI installer as a release asset

Alternatively, you can manually create a release on GitHub, and the workflow will build and attach the installer.

### Manual Workflow Trigger

You can also trigger the workflow manually from the GitHub Actions tab in your repository using the "workflow_dispatch" event.

## Option 3: Cross-Compilation from macOS (Advanced)

Cross-compiling to Windows from macOS is complex and requires additional setup. It's generally not recommended unless you have specific requirements.

### Prerequisites

1. Install the Windows target:
   ```bash
   rustup target add x86_64-pc-windows-msvc
   ```

2. Install `cargo-xwin` for cross-compilation:
   ```bash
   cargo install cargo-xwin
   ```

3. Set up the linker (complex, depends on your setup)

### Limitations

- Complex linker configuration
- May not work for all dependencies (some crates don't cross-compile well)
- Slower build times
- Not officially supported by Tauri for production builds

### Recommendation

If you need to build from macOS, consider:
- Using a Windows VM (VMware Fusion, Parallels, or VirtualBox)
- Using GitHub Actions (Option 2)
- Using a Windows machine (Option 1)

## Build Outputs

When you run `npm run tauri build`, Tauri will generate:

- **MSI Installer**: `src-tauri/target/release/bundle/msi/SSLBoard_<version>_x64_en-US.msi`
  - Windows standard installer format
  - Requires admin rights to install

- **NSIS Installer** (if configured): `src-tauri/target/release/bundle/nsis/SSLBoard_<version>_x64-setup.exe`
  - Customizable installer
  - Can be configured in `tauri.conf.json`

The MSI installer is the default for Windows builds.

## Signing Windows Installers (Optional but Recommended)

For distribution, you may want to code-sign your Windows installers. This requires:

1. A code signing certificate (from a Certificate Authority)
2. `signtool.exe` (comes with Windows SDK)

You can add a custom build script or use Tauri's build hooks to sign the installer automatically.

## Troubleshooting

### "linker 'link.exe' not found"
- Install Visual Studio Build Tools with C++ workload
- Or set up proper linker paths for cross-compilation

### "WebView2 not found"
- Ensure WebView2 Runtime is installed on the target Windows machine
- It comes pre-installed on Windows 10/11

### Build takes too long
- First build will compile all Rust dependencies (can take 10-30 minutes)
- Subsequent builds are much faster due to caching
- Consider using `cargo build --release` for testing before full Tauri build

