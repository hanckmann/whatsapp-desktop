use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder,
};

// Spoof a Chrome UA — WhatsApp Web blocks "headless" webview agents.
const WA_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/149.0.0.0 Safari/537.36";

// Additional navigator overrides injected before any page script runs.
const INIT_SCRIPT: &str = r#"
    (function() {
        const override = (obj, prop, value) => {
            try {
                Object.defineProperty(obj, prop, {
                    get: () => value,
                    configurable: false,
                });
            } catch (_) {}
        };
        override(navigator, 'vendor',   'Google Inc.');
        override(navigator, 'platform', 'Linux x86_64');
        override(navigator, 'appVersion',
            '5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) ' +
            'Chrome/149.0.0.0 Safari/537.36');
    })();
"#;

const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/32x32.png");

pub fn run() {
    // WebKitGTK on Wayland computes a broken device pixel ratio (dpr = -1/96)
    // from the screen DPI rather than the GDK widget scale, causing the CSS
    // viewport to report insane dimensions like -184128x-105024.
    // Forcing GDK_BACKEND=x11 uses XWayland's sane DPI path instead.
    if std::env::var("GDK_BACKEND").is_err() {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    // Disable DMA-BUF renderer — needed on NixOS/Wayland where the kernel DMA-BUF
    // GPU path is unavailable in a non-patched binary. Without this WebKit renders
    // a blank or malformed page. Does NOT disable CSS compositing.
    if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    // If launched outside the dev shell (e.g. from a .desktop file), ensure
    // GStreamer can find its plugins. The shell sets this correctly via flake.nix;
    // this only fills in a default when it's missing entirely.
    if std::env::var("GST_PLUGIN_SYSTEM_PATH_1_0").is_err() {
        // Attempt to locate plugins relative to the webkitgtk library path.
        // On NixOS this is a best-effort fallback; the flake shell is authoritative.
        if let Ok(out) = std::process::Command::new("pkg-config")
            .args(["--variable=pluginsdir", "gstreamer-plugins-base-1.0"])
            .output()
        {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path.is_empty() {
                std::env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", &path);
            }
        }
    }

    tauri::Builder::default()
        .setup(|app| {
            // ── Main window ──────────────────────────────────────────────
            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://web.whatsapp.com".parse()?),
            )
            .title("WhatsApp")
            .inner_size(1200.0, 820.0)
            .min_inner_size(600.0, 500.0)
            .user_agent(WA_USER_AGENT)
            .initialization_script(INIT_SCRIPT)
            .build()?;

            // ── System tray ──────────────────────────────────────────────
            let tray_icon = Image::from_bytes(TRAY_ICON_PNG)?;

            let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .title("WhatsApp Desktop")
                .tooltip("WhatsApp Desktop")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        toggle_window(app);
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        // Close button hides to tray instead of exiting.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running WhatsApp Desktop");
}

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
