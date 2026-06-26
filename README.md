# WhatsApp Desktop

A lightweight WhatsApp Web wrapper for Linux built with **Rust + Tauri v2**.

- Uses the OS-native **WebKitGTK** engine — no bundled Chromium, no Electron, no Node.js
- Spoofs a Chrome user agent so WhatsApp Web loads without complaints
- **Close → hides to system tray** (left-click tray icon to toggle, right-click for menu)
- Session persists between launches (cookies stored in `~/.local/share/app.whatsapp.desktop/`)

---

## Prerequisites (NixOS / Nix)

This project uses a **Nix flake dev shell** managed by **direnv** — no manual package installation needed.

```bash
# One-time: allow direnv to activate the shell for this directory
direnv allow
```

When you `cd` into the project, direnv automatically drops you into a shell with:
- `rustc` / `cargo` (stable, via `rust-overlay`)
- `cargo-tauri` (Tauri CLI v2 — no Node.js)
- `webkit2gtk 4.1`, `gtk3`, `libayatana-appindicator`, and all other native libs
- `python3` + `pillow` (for icon generation)

> **Non-NixOS users:** See the `buildInputs` list in `flake.nix` for the equivalent packages to install via your distro's package manager, then `cargo install tauri-cli`.

---

## Build

```bash
# 0. Enter the dev shell (automatic with direnv; or manually:)
nix develop

# 1. Generate icons (one-time)
python3 gen_icons.py

# 2. Run in development mode
cargo tauri dev

# 3. Build a release binary / AppImage / .deb
cargo tauri build
```

Binaries and packages are written to `src-tauri/target/release/bundle/`.

---

## Known Limitations

| Feature | Status |
|---|---|
| Text messaging | ✅ Works |
| Voice/video calls | ⚠️ May not work — WebRTC is incomplete in WebKitGTK |
| Desktop notifications | ✅ Handled by WebKit's Notification API |
| File upload / download | ✅ Works |
| Wayland global shortcut | ❌ OS limitation; use `cargo tauri dev -- --toggle` workaround |

---
