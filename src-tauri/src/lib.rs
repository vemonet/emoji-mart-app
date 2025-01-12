// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::process::Command;

use tauri::{command, AppHandle, Runtime};
// use tauri::{command, AppHandle, Manager, Runtime};

/// Check if x11 or wayland is used
fn xdg_session_type() -> String {
    env::var("XDG_SESSION_TYPE")
        .unwrap_or_else(|_| "wayland".to_string())
        .to_lowercase()
}

pub fn run() {
    // NOTE: tauri v2 clipboard seems to crash on wayland inside flatpak
    // If wayland: start ydotool for auto-paste
    // #[cfg(target_os = "linux")]
    // if xdg_session_type() == "wayland" {
    //     // ydotoold --socket-path="$HOME/.ydotool_socket" --socket-own="$(id -u):$(id -g)"
    //     match Command::new("ydotoold").spawn() {
    //         Ok(_child) => {}
    //         Err(error) => {
    //             eprintln!("[EmojiMart] ydotoold daemon failed to start: {}", error);
    //         }
    //     }
    // }

    tauri::Builder::default()
        // .plugin(tauri_plugin_single_instance::init())
        // .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        //     show_window(app);
        // }))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![trigger_paste])
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                // Close window whenever it loses focus
                if !focused && window.is_visible().unwrap() {
                    window.close().unwrap();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("[EmojiMart] Error while running tauri application");
}

// fn show_window(app: &AppHandle) {
//     let windows = app.webview_windows();
//     windows
//         .values()
//         .next()
//         .expect("Sorry, no window found")
//         .set_focus()
//         .expect("Can't Bring Window to Focus");
// }

/// Trigger auto-paste of emoji and close the app
#[command]
fn trigger_paste<R: Runtime>(
    emoji: &str,
    // keep: Option<bool>,
    app: AppHandle<R>,
) {
    // // Example copy to clipboard from rust
    // use tauri_plugin_clipboard_manager::ClipboardExt;
    // let clipboard_content = tauri_plugin_clipboard_manager::ClipKind::PlainText {
    //     label: Some("EmojiMart".to_string()),
    //     text: emoji.to_string(),
    // };
    // app.clipboard().write(clipboard_content).unwrap();

    // NOTE: app.exit needs to be done in a separate thread than copy to clipboard
    // Otherwise, the clipboard is lost after closing the app
    let emoji_owned = emoji.to_string();
    tauri::async_runtime::spawn(async move {
        // Auto-paste needs to be done in same thread as app.exit
        #[cfg(target_os = "linux")]
        {
            if xdg_session_type() == "x11" {
                // Paste on x11 with xdotool
                let result = Command::new("xdotool")
                    .arg("type")
                    .arg(&emoji_owned) // Pass owned string as argument
                    // .arg("key")
                    // .arg("ctrl+shift+v")
                    .spawn();
                if let Err(e) = result {
                    println!("[EmojiMart] xdotool paste command failed: {:?}", e);
                }
            }
            // else {
            //     // Paste on wayland with ydotool
            //     // Type don't work with emojis https://github.com/ReimuNotMoe/ydotool/issues/22
            //     // ydotool key 29:1 42:1 47:1 47:0 42:0 29:0
            //     // See also: https://github.com/obv-mikhail/InputBot/issues/4
            //     let result = Command::new("ydotool")
            //         .arg("key")
            //         .arg("29:1") // ctrl
            //         .arg("42:1") // shift
            //         .arg("47:1") // v
            //         .arg("47:0")
            //         .arg("42:0")
            //         .arg("29:0")
            //         .spawn();
            //     if let Err(e) = result {
            //         println!("[EmojiMart] ydotool paste command failed: {:?}", e);
            //     }
            // }
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
        // Wait 1s before closing to let the time to the user to paste the emoji because sometimes the clipboard is lost after closing the app
        app.exit(0);
    });
}

// Another option to auto-paste: Enigo, but bug with Tauri
// https://github.com/enigo-rs/enigo/issues/153
// https://github.com/tauri-apps/tauri/issues/6421
// use enigo::*;
// let mut enigo = Enigo::new();
// enigo.key_down(Key::Control);
// enigo.key_click(Key::Layout('v'));
// enigo.key_up(Key::Control);
