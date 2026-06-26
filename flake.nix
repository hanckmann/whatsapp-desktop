{
  description = "WhatsApp Desktop — Tauri v2 / WebKitGTK wrapper (no Electron, no Node.js)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};

        # Stable Rust toolchain — pin a version here if you need reproducibility
        rustToolchain = pkgs.rust-bin.stable.latest.default;

        # All native libs Tauri v2 needs on Linux
        nativeBuildInputs = with pkgs; [
          rustToolchain
          cargo-tauri # tauri-cli v2 — cargo tauri dev / build
          pkg-config
        ];

        buildInputs = with pkgs; [
          # WebKit / GTK stack
          webkitgtk_4_1
          gtk3
          glib
          cairo
          pango
          atk
          gdk-pixbuf
          # TLS support for WebKitGTK (provides libgiognutls.so GIO module)
          glib-networking
          # GStreamer — WebKitGTK uses these for its internal rendering pipeline
          # (appsink/appsrc from base, autoaudiosink from good). Missing these
          # causes rendering breakage even on non-media pages.
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
          gst_all_1.gst-plugins-good
          # Tray icon support
          libayatana-appindicator
          # Additional Tauri v2 deps
          librsvg
          libxkbcommon
          xdotool # for xdo (optional global shortcut helper)
          openssl
        ];

        # Dev-only helpers (not Tauri build deps)
        devTools = with pkgs; [
          python3Packages.pillow # for gen_icons.py
        ];
      in {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          packages = devTools;

          # WebKitGTK on Wayland computes dpr = -1/96 (broken, from screen DPI).
          # CSS viewport becomes -184128x-105024 — completely unusable.
          # x11 backend (XWayland) uses the correct DPI path and gives dpr=1.
          GDK_BACKEND = "x11";

          # WEBKIT_DISABLE_COMPOSITING_MODE intentionally NOT set — it breaks CSS
          # layout and rendering on Wayland by disabling the entire compositor.
          # WEBKIT_DISABLE_DMABUF_RENDERER=1 is the correct NixOS/Wayland workaround:
          # it skips the DMA-BUF GPU buffer path (which needs kernel driver support)
          # while leaving CSS compositing, transforms, and layer rendering intact.
          WEBKIT_DISABLE_DMABUF_RENDERER = "1";

          # Needed to run the built binary from within the dev shell —
          # Nix doesn't patch rpath on binaries built by cargo, so .so files
          # must be findable via LD_LIBRARY_PATH at runtime.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          # WebKitGTK loads TLS via GIO modules (libgiognutls.so from glib-networking).
          # Without this, all HTTPS requests fail with "TLS support is not available".
          GIO_EXTRA_MODULES = "${pkgs.glib-networking}/lib/gio/modules";

          # GStreamer plugin discovery for WebKitWebProcess (runs as a subprocess).
          # LD_LIBRARY_PATH alone is not enough — GStreamer scans these dirs for .so plugins.
          GST_PLUGIN_SYSTEM_PATH_1_0 = pkgs.lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0" (with pkgs.gst_all_1; [
            gstreamer
            gst-plugins-base
            gst-plugins-good
          ]);

          shellHook = ''
            # Tauri's AppImage bundler reads NIX_LDFLAGS as a list of file paths
            # and crashes on NixOS because the whole string isn't a valid path.
            # Unsetting it here is safe: rustc has already resolved the flags via
            # pkg-config; NIX_LDFLAGS is only consulted by the linker at compile time.
            unset NIX_LDFLAGS
            unset NIX_LDFLAGS_FOR_TARGET

            echo "WhatsApp Desktop dev-shell ready"
            echo "  rustc  : $(rustc --version)"
            echo "  cargo  : $(cargo --version)"
            echo "  tauri  : $(cargo tauri --version 2>/dev/null)"
            echo ""
            echo "  Generate icons  : python3 gen_icons.py"
            echo "  Run dev server  : cargo tauri dev"
            echo "  Build release   : cargo tauri build"
            echo "  Build .deb only : cargo tauri build --bundles deb"
          '';
        };
      }
    );
}
