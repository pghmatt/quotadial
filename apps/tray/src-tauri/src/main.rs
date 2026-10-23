// No console window on Windows for the release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    quotadial_tray::run();
}
