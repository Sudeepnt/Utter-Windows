# Utter for Windows

Private voice dictation for Windows. Hold a key, speak, let go: Utter writes
what you said into whatever app you're in.

- **Local.** Speech is transcribed on your PC. The first run downloads a
  speech model; after that nothing leaves your machine.
- **Fast.** Words appear as you speak, and paste the moment you let go.
- **Yours.** No account, no telemetry, no update server. Free and open source.

## Install

Download **UtterSetup-x64.exe** from the
[latest release](https://github.com/Sudeepnt/Utter-Windows/releases/latest)
and open it. On an ARM PC (such as a Snapdragon laptop), use
**UtterSetup-arm64.exe**. No administrator rights are needed.

The installer isn't code-signed yet, so Windows may say it protected your PC:
choose **More info**, then **Run anyway**.

Or install from PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -Command "iwr https://github.com/Sudeepnt/Utter-Windows/releases/latest/download/install-windows.ps1 -OutFile install-windows.ps1; ./install-windows.ps1"
```

Needs 64-bit Windows 10 or 11. x64 PCs need a 2013 or newer processor with
AVX2. On x64, speech is decoded on the GPU through Vulkan (any current NVIDIA,
AMD or Intel driver), falling back to the CPU without it. ARM64 uses the CPU.

## Use

- **Dictate:** hold **Ctrl+Space**, speak, release. Tap it twice to lock
  recording on, then press it again to finish.
- **Paste the last dictation again:** **Ctrl+Alt+V**.
- Change both, the microphone and the speech model in Utter's settings. Utter
  stays in the system tray; closing the window only hides it.

Utter installs into `%APPDATA%\utter` and adds itself to the Start menu,
**Settings › Apps** (for uninstalling) and the terminal's PATH as `utter`.
Settings and history live in `%APPDATA%\utter`, speech models and logs in
`%LOCALAPPDATA%\utter`; uninstalling keeps them.

## In-app pricing preview

Open **Pricing** in the sidebar. Utter is **Free right now** and requires no
purchase or paid license. Checkout is not active. Plan content lives in `src/rune_ui/pricing.rs`.

## Command line

`utter transcribe meeting.m4a` prints a recording's transcript. It reads WAV,
AIFF, CAF, FLAC, MP3, Ogg, WebM and MP4/M4A/MOV. The same `utter` controls the
running app: `utter status`, `utter dictation`, `utter settings`.

## Releases

GitHub builds Utter on its Windows machines
([.github/workflows/windows.yml](.github/workflows/windows.yml)). Every push
to `main` builds `utter.exe`, `utter.com` and the installers for x64 and ARM64,
tests the installer, and keeps them as the run's artifacts.

To publish a release:

1. Set the new version in `Cargo.toml` (`version = "3.4.1"`) and push it.
2. Tag that commit and push the tag:

   ```sh
   git tag v3.4.1
   git push origin v3.4.1
   ```

The workflow then publishes the installers, portable zips,
`install-windows.ps1` and `SHA256SUMS.txt` as a GitHub Release.

## Build from source on Windows

From the repository root in PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup-windows.ps1
cargo run -- app
```

The setup script installs Visual Studio's C++ build tools, CMake, Rust and
the Vulkan SDK if they're missing. Build the installer around a release build
with `scripts/build-windows-installer.ps1` (it installs Inno Setup when needed).

## Make it yours

- Colours: edit the three values at the top of
  `crates/utter-theme/tools/brand.py` and run it.
- Icons: replace `assets/icons/utter-logo.png`, then run `swift scripts/render-icons.swift`
  on a Mac to regenerate the app icons, including `resources/windows/utter.ico`.
- Fonts: `crates/utter-theme/assets/fonts` (Figtree and JetBrains Mono, SIL OFL).

## License

MIT. See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).


## Publishing updates

Packaged builds with `UTTER_GITHUB_UPDATES=1` check the public releases in
`Sudeepnt/Utter` ten seconds after startup, then every fifteen minutes. The
running version is compared numerically with the published desktop version;
older or identical versions are never offered as updates. Beta builds also see
published previews. Drafts, other platforms, unrelated URLs, and releases without
checksums are ignored.

When a newer installer exists, **Download update** stays in the sidebar and a
reminder returns ten minutes after dismissal. Clicking downloads the installer;
the user runs it and reopens Utter to complete the update. Unsigned previews do
not silently execute or replace an installed app. Existing signature requirements
for any future automatic self-replacement remain enforced.

For the next update, increment `Cargo.toml` and its root `Cargo.lock` entry in
both app repositories, build and verify installers, and publish a higher-version
release with installer assets and `SHA256SUMS.txt` to `Sudeepnt/Utter`. A code push
alone is not an installer release. Stable Mac distribution still requires a
Developer ID certificate and Apple notarization; publish unsigned test builds
only as clearly labelled previews.
