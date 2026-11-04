// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // D5-F7: in release builds, ignore WebView2 debugging and runtime overrides
    // from the environment (e.g. `--remote-debugging-port` in
    // WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS gives any local process full DOM
    // and IPC control). Removed before any thread or WebView starts.
    #[cfg(not(debug_assertions))]
    for var in [
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "WEBVIEW2_PIPE_FOR_SCRIPT_DEBUGGER",
        "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER",
    ] {
        // SAFETY: first thing in main, single-threaded: nothing else reads
        // or writes the environment concurrently.
        unsafe { std::env::remove_var(var) };
    }
    revelation_core::run();
}
