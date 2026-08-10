// Prevent a second console window from appearing beside the application in
// Windows release builds. Debug builds keep the console for diagnostics.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    bambook_lib::run();
}
