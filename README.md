# WhatsApp Desktop

A lightweight WhatsApp Web wrapper for Linux built with **Rust + Tauri v2**.

- Uses the OS-native **WebKitGTK** engine — no bundled Chromium, no Electron, no Node.js
- Spoofs a Chrome user agent so WhatsApp Web loads without complaints
- **Close → hides to system tray** (left-click tray icon to toggle, right-click for menu)
- Session persists between launches (cookies stored in `~/.local/share/app.whatsapp.desktop/`)

---

## Using in your Nix / NixOS configuration

Install the app from the flake without cloning this repo.

### Try it without installing

```bash
# Run once (build + launch, nothing persisted to your profile)
nix run github:hanckmann/whatsapp-desktop

# Drop into a shell with the binary on PATH
nix shell github:hanckmann/whatsapp-desktop
```

---

> **Tip:** Always add `inputs.whatsapp-desktop.inputs.nixpkgs.follows = "nixpkgs"` so the
> same nixpkgs revision is shared and the package is not built twice.

### NixOS — system-wide (`environment.systemPackages`)

```nix
# /etc/nixos/flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    whatsapp-desktop = {
      url = "github:hanckmann/whatsapp-desktop";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, whatsapp-desktop, ... }: {
    nixosConfigurations.myhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";   # or aarch64-linux
      modules = [
        ./configuration.nix
        ({ pkgs, ... }: {
          environment.systemPackages = [
            whatsapp-desktop.packages.${pkgs.system}.default
          ];
        })
      ];
    };
  };
}
```

Rebuild with:
```bash
sudo nixos-rebuild switch --flake /etc/nixos#myhostname
```

---

### home-manager on NixOS (module)

Add the input to your NixOS flake as shown above, then in your home-manager module:

```nix
home-manager.users.yourusername = { pkgs, ... }: {
  home.packages = [ whatsapp-desktop.packages.${pkgs.system}.default ];
};
```

---

### home-manager standalone (Ubuntu / non-NixOS)

Requires [Nix](https://nixos.org/download/) (multi-user install recommended) and
[home-manager](https://nix-community.github.io/home-manager/index.xhtml#sec-flakes-standalone)
configured with flakes.

```nix
# ~/.config/home-manager/flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    whatsapp-desktop = {
      url = "github:hanckmann/whatsapp-desktop";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, home-manager, whatsapp-desktop, ... }: {
    homeConfigurations."yourusername" = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;  # adjust arch if needed
      modules = [
        ./home.nix
        ({ pkgs, ... }: {
          home.packages = [
            whatsapp-desktop.packages.${pkgs.system}.default
          ];
        })
      ];
    };
  };
}
```

Rebuild with:
```bash
home-manager switch --flake ~/.config/home-manager#yourusername
```

The package installs a `.desktop` file so the app appears in your application launcher
after the next login (or run `update-desktop-database` manually).

> **Note:** On non-NixOS the binary uses Nix-store libraries (WebKitGTK, GTK3, GStreamer)
> via rpath — no system GTK libraries are required or affected.

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
