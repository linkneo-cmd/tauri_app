// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use vips;
fn main() {
    // VipsImage::init().expect("Failed to initialize Vips");
    vips::init(None).expect("Failed to initialize Vips");
    tauri_app_lib::run()
}
