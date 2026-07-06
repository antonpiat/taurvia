// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK on Linux (common with AppImage / NVIDIA) can show a blank window
    // while the page is loaded. See https://v2.tauri.app/develop/debug/linux-graphics/
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    aegis_desktop_lib::run()
}
