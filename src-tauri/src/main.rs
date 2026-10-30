// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use zoo_downloader_lib::native_messaging::is_native_messaging_mode;

fn main() {
    if is_native_messaging_mode() {
        // Chrome launched us as a native messaging host. Not implemented yet (review D-2):
        // the extension talks to the running app over its local HTTP endpoint instead.
        eprintln!("Native messaging mode is not implemented; start Zoo Downloader normally.");
        std::process::exit(1);
    }
    zoo_downloader_lib::run();
}
