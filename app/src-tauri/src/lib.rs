//! Revelation core (Tauri 2). Thin: read-only content queries in, plain data
//! out (`content`), and the one writer for user data (`user`).

pub mod books;
pub mod commands;
pub mod commentary;
pub mod content;
pub mod marks;
pub mod notes;
pub mod runs;
pub mod search;
pub mod user;

#[cfg(test)]
mod cmt_tests;
/// The synthetic placeholder fixture: tests only, never in a shipped binary.
#[cfg(test)]
pub mod fixture;
#[cfg(test)]
mod kjv_tests;
#[cfg(test)]
mod lex_tests;
#[cfg(test)]
mod pen_tests;
#[cfg(test)]
mod search_tests;

/// Expected SHA-256 of each content file, compiled in by build.rs from
/// `pipeline/tests/expected/output-sha256.txt` (pipeline/DESIGN.md 10.2).
pub mod expected {
    include!(concat!(env!("OUT_DIR"), "/expected_sha256.rs"));

    #[cfg(test)]
    mod tests {
        /// The compiled-in values are exactly the committed file's lines.
        #[test]
        fn compiled_in_hashes_equal_the_committed_file() {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../pipeline/tests/expected/output-sha256.txt");
            let e = revelation_content::hashfile::parse(&std::fs::read_to_string(path).unwrap())
                .unwrap();
            let got: Vec<(String, String)> = super::ALL
                .iter()
                .map(|(f, h)| (f.to_string(), h.to_string()))
                .collect();
            assert_eq!(got, e);
            assert_eq!(super::KJV_SQLITE, super::ALL[0].1);
            assert_eq!(super::CONTENT_SQLITE.len(), 64);
        }
    }
}

use std::sync::Mutex;
use tauri::Manager;

/// The command names, also listed in build.rs (permissions) and
/// capabilities/main.json. Keep the three in step.
pub fn run() {
    tauri::Builder::default()
        .plugin(window_state_plugin())
        .setup(|app| {
            let content = commands::load_content();
            let user = match app.path().app_data_dir() {
                Ok(dir) => commands::open_user(&dir),
                Err(_) => Err("no app data folder is available".into()),
            };
            app.manage(commands::AppState {
                content: Mutex::new(content),
                user: Mutex::new(user),
            });
            app.manage(commands::CloseGate::default());
            if let Some(w) = app.get_webview_window("main") {
                // a failure here only leaves the restored size as it was
                let _ = clamp_to_monitor(&w);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // W2-5: hold the close once so the page can save the notes first
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let gate = window.state::<commands::CloseGate>();
                let Some(n) = gate.request() else { return };
                let Some(view) = window.app_handle().get_webview_window(window.label()) else {
                    return;
                };
                api.prevent_close();
                let _ = view.eval("window.__revelationClose && window.__revelationClose()");
                let w = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(commands::CLOSE_WAIT);
                    if w.state::<commands::CloseGate>().timed_out(n) {
                        let _ = w.close();
                    }
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_books,
            commands::get_chapter,
            commands::get_strongs,
            commands::get_highlights,
            commands::set_highlight,
            commands::load_state,
            commands::save_state,
            commands::start_new_user_data,
            commands::search,
            commands::bookmarks_list,
            commands::bookmark_set,
            commands::bookmark_clear,
            commands::bookmark_restore,
            commands::get_commentary,
            commands::marks_for_chapter,
            commands::pen_apply,
            commands::pen_erase,
            commands::mark_remove,
            commands::mark_recolor,
            commands::marks_replace_chapter,
            commands::notes_list,
            commands::note_save,
            commands::app_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Revelation");
}

/// Window size, position and maximised state only (PLAN.md 7: restored
/// safely). The plugin restores the position only when the saved rectangle
/// intersects a connected monitor, otherwise the window opens centred. No
/// WebView permission is granted: the Rust side saves on close and restores on
/// creation, so the UI cannot call it.
fn window_state_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_window_state::StateFlags;
    tauri_plugin_window_state::Builder::new()
        .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
        .build()
}

/// D5-F3: the window-state plugin restores the saved size as is (it checks
/// only that the saved rectangle touches a monitor), so a size saved on a
/// larger monitor, or a tampered `.window-state.json`, could open a window far
/// bigger than the screen. Runs after the restore (config windows are built,
/// and restored, before `setup`). If the window doesn't fit the work area of
/// its monitor, it gets the default 1200 x 800 (or the work area, if smaller),
/// centred. A maximised window is left alone.
fn clamp_to_monitor(w: &tauri::WebviewWindow) -> tauri::Result<()> {
    if w.is_maximized()? {
        return Ok(());
    }
    let Some(m) = w.current_monitor()?.or(w.primary_monitor()?) else {
        return Ok(());
    };
    let area = m.work_area().size;
    let size = w.outer_size()?;
    if size.width <= area.width && size.height <= area.height {
        return Ok(());
    }
    let def: tauri::PhysicalSize<u32> =
        tauri::LogicalSize::new(1200.0, 800.0).to_physical(m.scale_factor());
    w.set_size(tauri::PhysicalSize::new(
        def.width.min(area.width),
        def.height.min(area.height),
    ))?;
    w.center()
}
